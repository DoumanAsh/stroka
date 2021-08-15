#[doc(hidden)]
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! unreach {
    () => ({
        unsafe {
            core::hint::unreachable_unchecked();
        }
    })
}

#[doc(hidden)]
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! unreach {
    () => ({
        unreachable!()
    })
}

pub trait MiniStr {
    fn as_str(&self) -> &str;
}

impl MiniStr for minivec::MiniVec<u8> {
    #[inline(always)]
    fn as_str(&self) -> &str {
        #[cfg(not(debug_assertions))]
        unsafe {
            core::str::from_utf8_unchecked(self.as_slice())
        }
        #[cfg(debug_assertions)]
        {
            core::str::from_utf8(self.as_slice()).expect("To contain UTF-8")
        }
    }
}
