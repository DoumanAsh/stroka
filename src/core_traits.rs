use crate::String;
use core::{fmt, hash};

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
            Self::Sso(ref sso) => Self::Sso(*sso),
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
        Some(core::cmp::Ord::cmp(self.as_str(), other.as_str()))
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
