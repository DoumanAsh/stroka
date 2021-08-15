//! `String` implementation optimized for small sized strings(at most length `mem::size_of::<usize>() * 2 - 2`)
//!
//! ## Features
//!
//! - `serde` - Enables `Serialize` and `Deserialize` implementations.
//! - `std` - Enables traits implementations dependent on `std`.
#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

extern crate alloc;

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "std")]
mod std;
mod utils;

use core::{ptr, mem, fmt, hash};

type HeapStr = minivec::MiniVec<u8>;
const SSO_MAX_SIZE: usize = mem::size_of::<HeapStr>() * 2 - 2;
type StrBuf = str_buf::StrBuf<{SSO_MAX_SIZE}>;

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

    ///Creates new string with provided initial value..
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
    pub const fn is_heap(&self) -> bool {
        match self {
            Self::Heap(_) => true,
            Self::Sso(_) => false,
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
            Self::Heap(ref mut heap) => unsafe {
                let text = core::str::from_utf8_unchecked_mut(heap.as_mut_slice());
                assert!(text.is_char_boundary(new_len));
                //in case of index out of boundary we panic above
                heap.set_len(new_len);
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
            Self::Heap(ref mut heap) => unsafe {
                let text = core::str::from_utf8_unchecked_mut(heap.as_mut_slice());
                let result = text.chars().last()?;
                heap.set_len(heap.len() - result.len_utf8());
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
        let len = self.len();
        match self {
            Self::Heap(ref mut heap) => heap.extend_from_slice(string.as_bytes()),
            Self::Sso(ref mut sso) => {
                if sso.remaining() < len {
                    let mut heap = self.assert_heap_from_sso(len + string.len());
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
    ///Decodes a UTF-16–encoded sequence into `String`.
    ///
    ///In case of invalid character, returns `DecodeUtf16Error`
    pub fn from_utf16(&mut self, utf16: &[u16]) -> Result<Self, core::char::DecodeUtf16Error> {
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
    pub fn from_utf16_lossy(&mut self, utf16: &[u16]) -> Self {
        let mut res = Self::with_capacity(utf16.len());
        for ch in char::decode_utf16(utf16.iter().cloned()) {
            res.push(ch.unwrap_or(core::char::REPLACEMENT_CHARACTER));
        }

        res
    }
}

impl From<char> for String {
    #[inline(always)]
    fn from(ch: char) -> String {
        let mut buf = [0u8; 4];
        Self::new_sso(ch.encode_utf8(&mut buf))
    }
}

impl From<&str> for String {
    #[inline(always)]
    fn from(s: &str) -> String {
        Self::new_str(s)
    }
}

impl From<&mut str> for String {
    #[inline(always)]
    fn from(s: &mut str) -> String {
        Self::new_str(s)
    }
}

impl From<&String> for String {
    #[inline(always)]
    fn from(s: &String) -> String {
        s.clone()
    }
}

impl From<alloc::boxed::Box<str>> for String {
    #[inline(always)]
    fn from(s: alloc::boxed::Box<str>) -> String {
        Self::new_str(&s)
    }
}

impl<'a> Extend<&'a char> for String {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a char>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.reserve(lower_bound);
        for ch in iter {
            self.push(*ch)
        }
    }
}

impl Extend<char> for String {
    #[inline]
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.reserve(lower_bound);
        for ch in iter {
            self.push(ch)
        }
    }
}

impl<'a> Extend<&'a str> for String {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}

impl Extend<alloc::boxed::Box<str>> for String {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = alloc::boxed::Box<str>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl<'a> Extend<alloc::borrow::Cow<'a, str>> for String {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = alloc::borrow::Cow<'a, str>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<String> for String {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl From<alloc::borrow::Cow<'_, str>> for String {
    #[inline(always)]
    fn from(s: alloc::borrow::Cow<'_, str>) -> String {
        Self::new_str(&s)
    }
}

impl core::str::FromStr for String {
    type Err = core::convert::Infallible;
    #[inline(always)]
    fn from_str(s: &str) -> Result<String, Self::Err> {
        Ok(Self::new_str(s))
    }
}

impl AsRef<[u8]> for String {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<str> for String {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsMut<str> for String {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl core::borrow::Borrow<str> for String {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl core::borrow::BorrowMut<str> for String {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl Clone for String {
    #[inline(always)]
    fn clone(&self) -> Self {
        match self {
            Self::Heap(ref heap) => Self::Heap(heap.clone()),
            Self::Sso(ref sso) => Self::Sso(sso.clone()),
        }
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Self::Heap(ref mut heap), Self::Heap(ref source)) => heap.clone_from(source),
            (this, _) => *this = source.clone(),
        }
    }
}

impl fmt::Debug for String {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for String {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Write for String {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}

impl hash::Hash for String {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        hash::Hash::hash(self.as_str(), hasher)
    }
}

impl Default for String {
    #[inline(always)]
    /// Creates an empty `String`.
    fn default() -> String {
        Self::new()
    }
}

impl core::ops::Deref for String {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl core::ops::DerefMut for String {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl PartialEq for String {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<str> for String {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<&str> for String {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<alloc::string::String> for String {
    #[inline(always)]
    fn eq(&self, other: &alloc::string::String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<alloc::borrow::Cow<'_, str>> for String {
    #[inline(always)]
    fn eq(&self, other: &alloc::borrow::Cow<'_, str>) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl Eq for String {
}

impl PartialEq<String> for &str {
    #[inline(always)]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(*self, other.as_str())
    }
}

impl PartialEq<String> for str {
    #[inline(always)]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl PartialEq<String> for alloc::string::String {
    #[inline(always)]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<String> for alloc::borrow::Cow<'_, str> {
    #[inline(always)]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl core::cmp::PartialOrd for String {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        core::cmp::PartialOrd::partial_cmp(self.as_str(), other.as_str())
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::lt(self.as_str(), other.as_str())
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::le(self.as_str(), other.as_str())
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::gt(self.as_str(), other.as_str())
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::ge(self.as_str(), other.as_str())
    }
}

impl core::cmp::Ord for String {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        core::cmp::Ord::cmp(self.as_str(), other.as_str())
    }
}

impl<'a> core::iter::FromIterator<&'a char> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> String {
        let mut buf = [0u8; 4];
        let mut res = String::new();
        for ch in iter {
            res.push_str(ch.encode_utf8(&mut buf));
        }
        res
    }
}

impl<'a> core::iter::FromIterator<&'a str> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> String {
        let mut res = String::new();
        for text in iter {
            res.push_str(text)
        }
        res
    }
}

impl core::iter::FromIterator<alloc::boxed::Box<str>> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = alloc::boxed::Box<str>>>(iter: I) -> String {
        let mut res = String::new();
        for text in iter {
            res.push_str(&text)
        }
        res
    }
}

impl core::iter::FromIterator<alloc::string::String> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = alloc::string::String>>(iter: I) -> String {
        let mut res = String::new();
        for text in iter {
            res.push_str(&text)
        }
        res
    }
}

impl core::iter::FromIterator<String> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> String {
        let mut iter = iter.into_iter();

        match iter.next() {
            None => String::new(),
            Some(mut res) => {
                for text in iter {
                    res.push_str(&text)
                }
                res
            }
        }
    }
}

impl core::ops::Add<&str> for String {
    type Output = String;

    #[inline(always)]
    fn add(mut self, other: &str) -> String {
        self.push_str(other);
        self
    }
}

impl core::ops::AddAssign<&str> for String {
    #[inline(always)]
    fn add_assign(&mut self, other: &str) {
        self.push_str(other);
    }
}

impl core::ops::Index<core::ops::Range<usize>> for String {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: core::ops::Range<usize>) -> &str {
        core::ops::Index::index(self.as_str(), index)
    }
}

impl core::ops::Index<core::ops::RangeTo<usize>> for String {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: core::ops::RangeTo<usize>) -> &str {
        core::ops::Index::index(self.as_str(), index)
    }
}

impl core::ops::Index<core::ops::RangeFrom<usize>> for String {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: core::ops::RangeFrom<usize>) -> &str {
        core::ops::Index::index(self.as_str(), index)
    }
}

impl core::ops::Index<core::ops::RangeFull> for String {
    type Output = str;

    #[inline(always)]
    fn index(&self, _: core::ops::RangeFull) -> &str {
        self.as_str()
    }
}

impl core::ops::Index<core::ops::RangeInclusive<usize>> for String {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: core::ops::RangeInclusive<usize>) -> &str {
        core::ops::Index::index(self.as_str(), index)
    }
}

impl core::ops::Index<core::ops::RangeToInclusive<usize>> for String {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: core::ops::RangeToInclusive<usize>) -> &str {
        core::ops::Index::index(self.as_str(), index)
    }
}

impl core::ops::IndexMut<core::ops::Range<usize>> for String {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::Range<usize>) -> &mut str {
        core::ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}

impl core::ops::IndexMut<core::ops::RangeTo<usize>> for String {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeTo<usize>) -> &mut str {
        core::ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}

impl core::ops::IndexMut<core::ops::RangeFrom<usize>> for String {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeFrom<usize>) -> &mut str {
        core::ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}

impl core::ops::IndexMut<core::ops::RangeFull> for String {
    #[inline(always)]
    fn index_mut(&mut self, _: core::ops::RangeFull) -> &mut str {
        self.as_mut_str()
    }
}

impl core::ops::IndexMut<core::ops::RangeInclusive<usize>> for String {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeInclusive<usize>) -> &mut str {
        core::ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}

impl core::ops::IndexMut<core::ops::RangeToInclusive<usize>> for String {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeToInclusive<usize>) -> &mut str {
        core::ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}
