// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(missing_docs, bad_style)]

use io::{self, ErrorKind};
use libc;

#[cfg(any(dox, target_os = "linux"))] pub use os::linux as platform;

#[cfg(all(not(dox), target_os = "android"))]   pub use os::android as platform;
#[cfg(all(not(dox), target_os = "bitrig"))]    pub use os::bitrig as platform;
#[cfg(all(not(dox), target_os = "dragonfly"))] pub use os::dragonfly as platform;
#[cfg(all(not(dox), target_os = "freebsd"))]   pub use os::freebsd as platform;
#[cfg(all(not(dox), target_os = "haiku"))]     pub use os::haiku as platform;
#[cfg(all(not(dox), target_os = "ios"))]       pub use os::ios as platform;
#[cfg(all(not(dox), target_os = "macos"))]     pub use os::macos as platform;
#[cfg(all(not(dox), target_os = "netbsd"))]    pub use os::netbsd as platform;
#[cfg(all(not(dox), target_os = "openbsd"))]   pub use os::openbsd as platform;
#[cfg(all(not(dox), target_os = "solaris"))]   pub use os::solaris as platform;
#[cfg(all(not(dox), target_os = "emscripten"))] pub use os::emscripten as platform;
#[cfg(all(not(dox), target_os = "fuchsia"))]   pub use os::fuchsia as platform;
#[cfg(all(not(dox), target_os = "l4re"))]      pub use os::linux as platform;
#[cfg(all(not(dox), target_os = "horizon"))]      pub use os::horizon as platform;

pub use self::rand::hashmap_random_keys;
pub use libc::strlen;

#[macro_use]
pub mod weak;

pub mod args;
pub mod android;
#[cfg(feature = "backtrace")]
pub mod backtrace;
pub mod cmath;
pub mod condvar;
pub mod env;
pub mod ext;
pub mod fast_thread_local;
pub mod fd;
pub mod fs;
pub mod memchr;
pub mod mutex;
#[cfg(not(target_os = "l4re"))]
pub mod net;
#[cfg(target_os = "l4re")]
mod l4re;
#[cfg(target_os = "l4re")]
pub use self::l4re::net;
pub mod os;
pub mod os_str;
pub mod path;
pub mod pipe;
pub mod process;
pub mod rand;
pub mod rwlock;
pub mod stack_overflow;
pub mod thread;
pub mod thread_local;
pub mod time;
pub mod stdio;

#[cfg(not(test))]
pub fn init() {
    // By default, some platforms will send a *signal* when an EPIPE error
    // would otherwise be delivered. This runtime doesn't install a SIGPIPE
    // handler, causing it to kill the program, which isn't exactly what we
    // want!
    //
    // Hence, we set SIGPIPE to ignore when the program starts up in order
    // to prevent this problem.
    unsafe {
        reset_sigpipe();
    }

    #[cfg(not(any(target_os = "emscripten", target_os = "fuchsia")))]
    unsafe fn reset_sigpipe() {
        assert!(signal(0 as _, libc::SIG_IGN) != libc::SIG_ERR);
    }
    #[cfg(any(target_os = "emscripten", target_os = "fuchsia"))]
    unsafe fn reset_sigpipe() {}
}

#[cfg(target_os = "android")]
pub use sys::android::signal;
#[cfg(not(target_os = "android"))]
pub use libc::signal;

pub fn unsupported<T>() -> io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> io::Error {
    io::Error::new(io::ErrorKind::Other,
                   "operation not supported on 3DS yet")
}

#[derive(Copy,Clone)]
pub enum Void {}

pub fn decode_error_kind(errno: i32) -> ErrorKind {
    match errno as libc::c_int {
        libc::ECONNREFUSED => ErrorKind::ConnectionRefused,
        libc::ECONNRESET => ErrorKind::ConnectionReset,
        libc::EPERM | libc::EACCES => ErrorKind::PermissionDenied,
        libc::EPIPE => ErrorKind::BrokenPipe,
        libc::ENOTCONN => ErrorKind::NotConnected,
        libc::ECONNABORTED => ErrorKind::ConnectionAborted,
        libc::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
        libc::EADDRINUSE => ErrorKind::AddrInUse,
        libc::ENOENT => ErrorKind::NotFound,
        libc::EINTR => ErrorKind::Interrupted,
        libc::EINVAL => ErrorKind::InvalidInput,
        libc::ETIMEDOUT => ErrorKind::TimedOut,
        libc::EEXIST => ErrorKind::AlreadyExists,

        // These two constants can have the same value on some systems,
        // but different values on others, so we can't use a match
        // clause
        x if x == libc::EAGAIN || x == libc::EWOULDBLOCK =>
            ErrorKind::WouldBlock,

        _ => ErrorKind::Other,
    }
}

#[doc(hidden)]
pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

pub fn cvt_r<T, F>(mut f: F) -> io::Result<T>
    where T: IsMinusOne,
          F: FnMut() -> T
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

// On Unix-like platforms, libc::abort will unregister signal handlers
// including the SIGABRT handler, preventing the abort from being blocked, and
// fclose streams, with the side effect of flushing them so libc bufferred
// output will be printed.  Additionally the shell will generally print a more
// understandable error message like "Abort trap" rather than "Illegal
// instruction" that intrinsics::abort would cause, as intrinsics::abort is
// implemented as an illegal instruction.
pub unsafe fn abort_internal() -> ! {
    ::libc::abort()
}
