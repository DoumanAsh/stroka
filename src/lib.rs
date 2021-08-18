//! `String` implementation optimized for small sized strings(at most length `mem::size_of::<usize>() * 2 - 2`)
//!
//! ## Features
//!
//! - `serde` - Enables `Serialize` and `Deserialize` implementations.
//! - `std` - Enables traits implementations dependent on `std`.
//!
//! ## Missing functions
//!
//! - `String::from_utf8` - due to `minivec` yet to be stable.
//! - `String::from_utf8_unchecked` - due to `minivec` yet to be stable.
//! - `String::into_bytes` - due to `minivec` yet to be stable.
//! - Unstable functions of Vec - due to them being potentially changed.
//! - `String::from_raw_parts` - cannot be implemented due to internal structure.

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

extern crate alloc;

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "std")]
mod std;
mod core_traits;
mod str_ext;
pub use str_ext::StrExt;
mod utils;
use utils::MiniStr;

use core::{ptr, mem};

type HeapStr = minivec::MiniVec<u8>;
const SSO_MAX_SIZE: usize = mem::size_of::<HeapStr>() * 2 - 2;
type StrBuf = str_buf::StrBuf<{SSO_MAX_SIZE}>;

#[inline(always)]
unsafe fn insert_bytes_into(ptr: *mut u8, len: usize, idx: usize, bytes: &[u8]) {
    let bytes_len = bytes.len();
    ptr::copy(ptr.add(idx), ptr.add(idx + bytes_len), len - idx);
    ptr::copy(bytes.as_ptr(), ptr.add(idx), bytes_len);
}

//verifies validity of range and returns its length
fn assert_range_len(this: &str, start: core::ops::Bound<&usize>, end: core::ops::Bound<&usize>) -> (usize, usize, usize) {
    let start = match start {
        core::ops::Bound::Included(n) => {
            assert!(this.is_char_boundary(*n));
            *n
        },
        core::ops::Bound::Excluded(n) => {
            let n = n.saturating_add(1);
            assert!(this.is_char_boundary(n));
            n
        },
        core::ops::Bound::Unbounded => 0,
    };
    let end = match end {
        core::ops::Bound::Included(n) => {
            let n = n.saturating_add(1);
            assert!(this.is_char_boundary(n));
            n
        },
        core::ops::Bound::Excluded(n) => {
            assert!(this.is_char_boundary(*n));
            *n
        },
        core::ops::Bound::Unbounded => this.len()
    };

    if start > end {
        panic!("start '{}' is greater than end '{}'", start, end);
    }

    (start, end, end - start)
}

///`String`, similar to that in `std`, but optimized with SSO (small string optimization).
///
///Its size is limited to 2 words (i.e. `mem::size_of::<usize>()`).
///For that purpose static buffer size is `mem::size_of::<usize>() * 2 - 2`
///`2` bytes are removed in order to fit buffer's length and variant discriminant.
///
///On 64bit platform it means buffer size is `14` bytes which is sufficient to hold small strings.
///For obvious reasons 32bit targets have smaller buffer size of `6` bytes.
///
///When string's content overflows static buffer, its content is moved onto heap.
///Clearing/shrinking capacity will no longer switch back at this point.
pub enum String {
    #[doc(hidden)]
    Heap(HeapStr),
    #[doc(hidden)]
    Sso(StrBuf),
}

impl String {
    ///Creates new empty instance.
    #[inline]
    pub const fn new() -> Self {
        Self::Sso(StrBuf::new())
    }

    ///Creates new string with provided initial value.
    #[inline]
    pub fn new_str(text: &str) -> Self {
        match StrBuf::from_str_checked(text) {
            Ok(sso) => Self::Sso(sso),
            Err(_) => Self::Heap(text.into()),
        }
    }

    ///Creates new static string.
    ///
    ///Panics in case of buffer overflow.
    #[inline]
    pub const fn new_sso(text: &str) -> Self {
        Self::Sso(StrBuf::from_str(text))
    }

    ///Creates new empty instance with specified capacity.
    ///
    ///If `capacity` is greater than static buffer can hold,
    ///`String` immediately allocates storage on heap.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= StrBuf::capacity() {
            Self::new()
        } else {
            Self::Heap(HeapStr::with_capacity(capacity))
        }
    }

    #[inline]
    ///Returns whether string is heap allocated.
    pub const fn is_alloc(&self) -> bool {
        match self {
            Self::Heap(_) => true,
            Self::Sso(_) => false,
        }
    }

    #[inline]
    ///Sets string length, ignoring whathever capacity is available.
    ///
    ///User is responsible to guarantee that `0..new_len` is valid string
    pub unsafe fn set_len(&mut self, new_len: usize) {
        match self {
            Self::Heap(ref mut buf) => buf.set_len(new_len ),
            Self::Sso(ref mut string) => string.set_len(new_len as u8),
        }
    }

    #[inline]
    ///Returns length of the underlying bytes storage.
    pub fn len(&self) -> usize {
        match self {
            Self::Heap(ref buf) => buf.len(),
            Self::Sso(ref string) => string.len(),
        }
    }

    #[inline(always)]
    fn assert_heap_from_sso(&self, capacity: usize) -> HeapStr {
        if let Self::Sso(ref buf) = self {
            let mut heap = HeapStr::with_capacity(capacity);

            unsafe {
                ptr::copy_nonoverlapping(buf.as_ptr() as *const u8, heap.as_mut_ptr(), buf.len());
                heap.set_len(buf.len());
            }

            heap
        } else {
            unreach!()
        }
    }

    ///Reserves additional space to store at least `additional` number of elements.
    ///
    ///The capacity may be increased by more than additional bytes if it chooses, to prevent
    ///frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();
        let required = self.len() + additional;

        if required <= capacity {
            return;
        }

        match self {
            Self::Sso(_) => *self = Self::Heap(self.assert_heap_from_sso(required)),
            Self::Heap(ref mut string) => string.reserve(additional),
        }
    }

    ///Reserves additional space to store exactly `additional` number of elements.
    pub fn reserve_exact(&mut self, additional: usize) {
        let capacity = self.capacity();
        let required = self.len() + additional;

        if required <= capacity {
            return;
        }

        match self {
            Self::Sso(_) => *self = Self::Heap(self.assert_heap_from_sso(required)),
            Self::Heap(ref mut string) => string.reserve_exact(additional),
        }
    }

    #[inline]
    ///Shrinks the capacity of this `String` to match its length.
    ///
    ///Does nothing while string is not heap allocated.
    pub fn shrink_to_fit(&mut self) {
        if let Self::Heap(ref mut heap) = self {
            heap.shrink_to_fit();
        }
    }

    #[inline]
    ///Returns `capacity`, indicating number of elements, that can be stored by underlying storage.
    pub fn capacity(&self) -> usize {
        match self {
            Self::Heap(ref heap) => heap.capacity(),
            Self::Sso(_) => StrBuf::capacity(),
        }
    }

    #[inline(always)]
    ///Returns pointer to the underlying storage.
    pub fn as_ptr(&self) -> *const u8 {
        match self {
            Self::Heap(ref heap) => heap.as_ptr(),
            Self::Sso(ref sso) => sso.as_ptr(),
        }
    }

    #[inline]
    ///Returns mutable pointer to the underlying storage.
    ///
    ///Note that write to such pointer is unsafe not only because of
    ///potential overflow, but the fact that user must write valid utf-8 byte sequence.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        match self {
            Self::Heap(ref mut heap) => heap.as_mut_ptr(),
            Self::Sso(ref mut sso) => sso.as_mut_ptr(),
        }
    }

    #[inline(always)]
    ///Access content of string as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Heap(ref heap) => heap.as_slice(),
            Self::Sso(ref sso) => sso.as_slice(),
        }
    }

    #[inline(always)]
    ///Access content of string as mutable bytes.
    ///
    ///Note that modifying this slice is `unsafe` hence this function is marked unsafe
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        match self {
            Self::Heap(ref mut heap) => heap.as_mut_slice(),
            Self::Sso(ref mut sso) => sso.as_mut_slice(),
        }
    }


    #[inline(always)]
    ///Gets string slice.
    pub fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(self.as_bytes())
        }
    }

    #[inline(always)]
    ///Gets mutable string slice.
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe {
            core::str::from_utf8_unchecked_mut(self.as_mut_bytes())
        }
    }

    #[inline]
    ///Clears content of string, leaving allocated storage intact.
    pub fn clear(&mut self) {
        match self {
            Self::Heap(ref mut heap) => heap.clear(),
            Self::Sso(ref mut sso) => sso.clear(),
        }
    }

    #[inline]
    ///Shortens `String` to the specified length.
    ///
    ///If `new_len` is greater than the string's current length, this has no
    ///effect.
    ///
    ///Note that this method has no effect on the allocated capacity
    ///of the string
    ///
    ///# Panics
    ///
    ///Panics if `new_len` does not lie on a `char` boundary.
    pub fn truncate(&mut self, new_len: usize) {
        match self {
            Self::Heap(ref mut heap) => {
                assert!(heap.as_str().is_char_boundary(new_len));
                //in case of index out of boundary we panic above
                unsafe {
                    heap.set_len(new_len);
                }
            },
            Self::Sso(ref mut sso) => {
                assert!(sso.is_char_boundary(new_len));
                //in case of index out of boundary we panic above
                unsafe {
                    sso.set_len(new_len as u8);
                }
            },
        }
    }

    #[inline(always)]
    ///Returns whether string is empty or not.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Heap(ref heap) => heap.len() == 0,
            Self::Sso(ref sso) => sso.len() == 0,
        }
    }

    #[inline]
    ///Removes the last character from the string and returns it, if there is any.
    pub fn pop(&mut self) -> Option<char> {
        let result = match self {
            Self::Heap(ref mut heap) => {
                let result = heap.as_str().chars().last()?;
                unsafe {
                    heap.set_len(heap.len() - result.len_utf8());
                }
                result
            },
            Self::Sso(ref mut sso) => {
                let result = sso.as_str().chars().last()?;
                unsafe {
                    sso.set_len(sso.len() as u8 - result.len_utf8() as u8);
                }
                result
            }
        };

        Some(result)
    }

    #[inline]
    ///Removes character at the specified `idx`
    ///
    ///This is an *O*(*n*) operation, as it requires copying every element in the buffer.
    ///
    ///# Panics
    ///
    ///If `idx` is larger than or equal to the `String`'s length, or if it does not lie on a [`char`] boundary.
    pub fn remove(&mut self, idx: usize) -> char {
        let result = match self {
            Self::Heap(ref mut heap) => {
                let ch = match heap.as_str()[idx..].chars().next() {
                    Some(ch) => ch,
                    None => panic!("cannot remove a char from the end of a string")
                };

                let next = idx + ch.len_utf8();
                let len = heap.len();

                unsafe {
                    ptr::copy(heap.as_ptr().add(next), heap.as_mut_ptr().add(idx), len - next);
                    heap.set_len(len - (next - idx));
                }
                ch
            },
            Self::Sso(ref mut sso) => {
                let ch = match sso.as_str()[idx..].chars().next() {
                    Some(ch) => ch,
                    None => panic!("cannot remove a char from the end of a string")
                };

                let next = idx + ch.len_utf8();
                let len = sso.len();
                unsafe {
                    ptr::copy(sso.as_ptr().add(next), sso.as_mut_ptr().add(idx), len - next);
                    sso.set_len(len as u8 - (next as u8 - idx as u8));
                }
                ch
            }
        };

        result
    }

    ///Retains only the characters specified by the predicate.
    ///
    ///In other words, remove all characters `c` such that `cb(c)` returns `false`.
    ///This method operates in place, visiting each character exactly once in the
    ///original order, and preserves the order of the retained characters.
    pub fn retain<F: FnMut(char) -> bool>(&mut self, mut cb: F) {
        #[inline(always)]
        fn get_char_from_slice(slice: &[u8]) -> Option<char> {
            unsafe { core::str::from_utf8_unchecked(slice) }.chars().next()
        }

        macro_rules! impl_retain {
            ($storage:expr, $typ:ident) => {
                struct LenSetter<'a> {
                    storage: &'a mut $typ,
                    idx: usize,
                    del_bytes: usize,
                }

                //It is highly unlikely to be needed, but just in case
                impl<'a> Drop for LenSetter<'a> {
                    #[inline(always)]
                    fn drop(&mut self) {
                        let new_len = self.idx - self.del_bytes;
                        debug_assert!(new_len <= self.storage.len());
                        unsafe {
                            self.storage.set_len(new_len as _);
                        }

                    }
                }

                let mut guard = LenSetter {
                    storage: $storage,
                    idx: 0,
                    del_bytes: 0,
                };
                let len = guard.storage.len();

                while let Some(ch) = guard.storage.as_slice().get(guard.idx..len).and_then(get_char_from_slice) {
                    let ch_len = ch.len_utf8();

                    if !cb(ch) {
                        guard.del_bytes += ch_len;
                    } else if guard.del_bytes > 0 {
                        unsafe {
                            ptr::copy(guard.storage.as_ptr().add(guard.idx),
                                      guard.storage.as_mut_ptr().add(guard.idx - guard.del_bytes),
                                      ch_len);
                        }
                    }

                    guard.idx += ch_len;
                }
            }
        }

        match self {
            Self::Heap(ref mut heap) => {
                impl_retain!(heap, HeapStr);
            },
            Self::Sso(ref mut sso) => {
                impl_retain!(sso, StrBuf);
            }
        }
    }

    #[inline(always)]
    ///Appends given `ch` at the end of the string.
    pub fn push(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        let res = ch.encode_utf8(&mut buf);

        self.push_str(res)
    }

    #[inline]
    ///Appends given `string` at the end.
    pub fn push_str(&mut self, string: &str) {
        match self {
            Self::Heap(ref mut heap) => heap.extend_from_slice(string.as_bytes()),
            Self::Sso(ref mut sso) => {
                let len = sso.len();
                let string_len = string.len();
                if sso.remaining() < string_len {
                    let mut heap = self.assert_heap_from_sso(len + string_len);
                    heap.extend_from_slice(string.as_bytes());
                    *self = Self::Heap(heap);
                } else {
                    unsafe {
                        sso.push_str_unchecked(string);
                    }
                }
            }
        }
    }

    #[inline]
    ///Inserts `char` at the given position
    ///# Panics
    ///
    ///Panics if `new_len` does not lie on a `char` boundary.
    pub fn insert(&mut self, idx: usize, ch: char) {
        let mut bits = [0; 4];
        self.insert_str(idx, ch.encode_utf8(&mut bits))
    }

    #[inline]
    ///Inserts `str` at the given position
    ///
    ///# Panics
    ///
    ///Panics if `new_len` does not lie on a `char` boundary.
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        let string_len = string.len();
        match self {
            Self::Heap(ref mut heap) => {
                assert!(heap.as_str().is_char_boundary(idx));

                heap.reserve(string_len);
                unsafe {
                    insert_bytes_into(heap.as_mut_ptr(), heap.len(), idx, string.as_bytes());
                    heap.set_len(heap.len() + string_len);
                }
            },
            Self::Sso(ref mut sso) => {
                assert!(sso.is_char_boundary(idx));

                let len = sso.len();
                if sso.remaining() < string_len {
                    let mut heap = self.assert_heap_from_sso(len + string_len);
                    unsafe {
                        insert_bytes_into(heap.as_mut_ptr(), heap.len(), idx, string.as_bytes());
                        heap.set_len(len + string_len);
                    }
                    *self = Self::Heap(heap);
                } else {
                    unsafe {
                        insert_bytes_into(sso.as_mut_ptr(), len, idx, string.as_bytes());
                        sso.set_len(len as u8 + string_len as u8);
                    }
                }
            }
        }
    }

    ///Removes the specified within the string.
    ///
    ///## Note
    ///
    ///This API is not part of `String` original API.
    ///
    ///## Panics
    ///
    ///Panics if the starting point or end point do not lie on a [`char`] boundary, or if they're out of bounds.
    pub fn remove_range<R: core::ops::RangeBounds<usize>>(&mut self, range: R) {
        //Defense against retarded impl
        let range_start = range.start_bound();
        let range_end = range.end_bound();

        match self {
            Self::Heap(ref mut heap) => {
                let (start, end, range_size) = assert_range_len(heap.as_str(), range_start, range_end);
                unsafe {
                    ptr::copy(heap.as_ptr().add(end), heap.as_mut_ptr().add(start), heap.len() - start - range_size);
                    heap.set_len(heap.len() - range_size);
                }
            },
            Self::Sso(ref mut sso) => {
                let (start, end, range_size) = assert_range_len(sso.as_str(), range_start, range_end);
                unsafe {
                    ptr::copy(sso.as_ptr().add(end), sso.as_mut_ptr().add(start), sso.len() - start - range_size);
                    sso.set_len(sso.len() as u8 - range_size as u8);
                }
            }
        }
    }

    #[inline]
    ///Removes the specified range in the string, and replaces it with the given string.
    ///The given string doesn't need to be the same length as the range.
    ///
    ///## Panics
    ///
    ///Panics if the starting point or end point do not lie on a [`char`] boundary, or if they're out of bounds.
    pub fn replace_range<R: core::ops::RangeBounds<usize>>(&mut self, range: R, string: &str) {
        //Defense against retarded impl
        let range_start = range.start_bound();
        let range_end = range.end_bound();

        match self {
            Self::Heap(ref mut heap) => {
                let text = heap.as_str();
                let (start, _, range_size) = assert_range_len(text, range_start, range_end);
                if range_size == string.len() {
                    unsafe {
                        ptr::copy(string.as_ptr(), heap.as_mut_ptr().add(start), range_size);
                    }
                } else {
                    heap.splice((range_start, range_end), string.bytes());
                }
            },
            Self::Sso(ref mut sso) => {
                let (start, end, range_size) = assert_range_len(sso.as_str(), range_start, range_end);
                let required = sso.len() - range_size + string.len();
                if StrBuf::capacity() < required {
                    let mut heap = self.assert_heap_from_sso(required);
                    heap.splice((range_start, range_end), string.bytes());
                    *self = Self::Heap(heap);
                } else {
                    if range_size == string.len() {
                        unsafe {
                            ptr::copy(string.as_ptr(), sso.as_mut_ptr().add(start), range_size);
                        }
                    } else  {
                        if let Some(diff) = range_size.checked_sub(string.len()) {
                            //range_size > string.len()
                            unsafe {
                                ptr::copy(sso.as_ptr().add(end), sso.as_mut_ptr().add(start + diff), sso.len() - diff);
                            }
                        } else {
                            let diff = string.len() - range_size;
                            if let Some(len_diff) = sso.len().checked_sub(diff) {
                                unsafe {
                                    ptr::copy(sso.as_ptr().add(start + diff), sso.as_mut_ptr().add(end + diff), len_diff);
                                }
                            }
                        }

                        unsafe {
                            ptr::copy(string.as_ptr(), sso.as_mut_ptr().add(start), string.len());
                            sso.set_len(required as _);
                        }
                    }
                }
            },
        }
    }

    #[inline]
    ///Decodes a UTF-16–encoded sequence into `String`.
    ///
    ///In case of invalid character, returns `DecodeUtf16Error`
    pub fn from_utf16(utf16: &[u16]) -> Result<Self, core::char::DecodeUtf16Error> {
        let mut res = Self::with_capacity(utf16.len());
        for ch in char::decode_utf16(utf16.iter().cloned()) {
            res.push(ch?);
        }

        Ok(res)
    }

    #[inline]
    ///Decodes a UTF-16–encoded sequence into `String`.
    ///
    ///In case of invalid character, replaces it with [REPLACEMENT_CHARACTER](https://doc.rust-lang.org/core/char/constant.REPLACEMENT_CHARACTER.html)
    pub fn from_utf16_lossy(utf16: &[u16]) -> Self {
        let mut res = Self::with_capacity(utf16.len());
        for ch in char::decode_utf16(utf16.iter().cloned()) {
            res.push(ch.unwrap_or(core::char::REPLACEMENT_CHARACTER));
        }

        res
    }
}
