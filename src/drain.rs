use crate::String;
use core::{fmt, ptr};
use core::str::Chars;
use core::iter::{FusedIterator, DoubleEndedIterator};

///Draining iterator over `String`
pub struct Drain<'a> {
    pub(crate) string: *mut String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) chars: Chars<'a>
}

impl<'a> Drain<'a> {
    #[inline]
    ///Returns the remaining sub-string of this iterator.
    pub fn as_str(&self) -> &str {
        self.chars.as_str()
    }
}

impl Iterator for Drain<'_> {
    type Item = char;

    #[inline(always)]
    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }

    #[inline(always)]
    fn last(mut self) -> Option<char> {
        self.chars.next_back()
    }
}

impl DoubleEndedIterator for Drain<'_> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<char> {
        self.chars.next_back()
    }
}

impl FusedIterator for Drain<'_> {}

impl<'a> Drop for Drain<'a> {
    fn drop(&mut self) {
        let this = unsafe {
            &mut *(self.string)
        };

        let range_size = self.end - self.start;
        match this {
            String::Heap(ref mut heap) => {
                unsafe {
                    let ptr = heap.as_mut_ptr();
                    ptr::copy(ptr.add(self.end), ptr.add(self.start), heap.len() - self.start - range_size);
                    heap.set_len(heap.len() - range_size);
                }
            },
            String::Sso(ref mut sso) => {
                unsafe {
                    let ptr = sso.as_mut_ptr();
                    ptr::copy(ptr.add(self.end), ptr.add(self.start), sso.len() - self.start - range_size);
                    sso.set_len(sso.len() - range_size);
                }
            },
        }
    }
}

impl fmt::Debug for Drain<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Drain").field(&self.as_str()).finish()
    }
}

unsafe impl Sync for Drain<'_> {}
unsafe impl Send for Drain<'_> {}
