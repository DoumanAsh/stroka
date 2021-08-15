extern crate std;

use crate::String;

use alloc::vec;
use std::io;
use std::net::{ToSocketAddrs, SocketAddr};
use std::ffi::OsStr;
use std::path::Path;

impl ToSocketAddrs for String {
    type Iter = vec::IntoIter<SocketAddr>;
    #[inline(always)]
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        self.as_str().to_socket_addrs()
    }
}

impl AsRef<OsStr> for String {
    #[inline(always)]
    fn as_ref(&self) -> &OsStr {
        self.as_str().as_ref()
    }
}

impl AsRef<Path> for String {
    #[inline(always)]
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}
