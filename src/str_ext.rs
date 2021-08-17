use core::ptr;
use crate::String;

///Extension trait to override methods that returns std's String
pub trait StrExt {
    ///Creates a new `String` by repeating a string `times`.
    ///
    ///## Panics
    ///
    ///This function will panic if the capacity would overflow.
    fn repeat(&self, times: usize) -> String;

    ///Returns a copy of this string where each character is mapped to its
    ///ASCII upper case equivalent.
    ///
    ///ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    ///but non-ASCII letters are unchanged.
    fn to_ascii_uppercase(&self) -> String;

    ///Returns a copy of this string where each character is mapped to its ASCII lower case equivalent.
    ///
    ///ASCII letters ‘A’ to ‘Z’ are mapped to ‘a’ to ‘z’, but non-ASCII letters are unchanged.
    fn to_ascii_lowercase(&self) -> String;
}

impl StrExt for str {
    #[inline]
    fn repeat(&self, times: usize) -> String {
        let len = self.len();
        let required = match len.checked_mul(times) {
            Some(required) => required,
            None => panic!("repeat capacity overflow"),
        };

        if required > 0 {
            let mut result = String::with_capacity(required);
            let result_ptr = result.as_mut_ptr();

            for idx in 0..times {
                unsafe {
                    ptr::copy_nonoverlapping(self.as_ptr(), result_ptr.add(len * idx), len);
                }
            }

            unsafe {
                result.set_len(self.len() * times);
            }
            result
        } else {
            String::new()
        }
    }

    #[inline]
    fn to_ascii_uppercase(&self) -> String {
        let mut res = String::new_str(self);
        res.make_ascii_uppercase();
        res
    }

    #[inline]
    fn to_ascii_lowercase(&self) -> String {
        let mut res = String::new_str(self);
        res.make_ascii_lowercase();
        res
    }

}
