use std::fmt::Display;
// Taken from project uname-rs
// https://github.com/caverym/uname-rs
use std::io::{Error, ErrorKind, Result};
use std::os::raw::{c_char, c_int};

macro_rules! returnerr {
    ($e:expr) => {
        Err(Error::new(ErrorKind::Other, $e))
    };
}

/// Raw `utsname` struct from `/usr/include/sys/utsname.h` C header.
#[repr(C)]
#[derive(Copy, Clone)]
struct utsname {
    pub sysname: [c_char; 65usize],
    pub nodename: [c_char; 65usize],
    pub release: [c_char; 65usize],
    pub version: [c_char; 65usize],
    pub machine: [c_char; 65usize],
    pub _domainname: [c_char; 65usize],
}

extern "C" {
    fn uname(__name: *mut utsname) -> c_int;
}

/// Safe implementation of `sys/utsname.h` header.
pub struct Uname {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
    pub domainname: String,
}

impl Uname {
    /// Collects and converts system information into Uname struct.
    /// Returns `Err` on failure, `Ok` on success.
    pub fn new() -> Result<Self> {
        let mut raw: utsname = unsafe { std::mem::zeroed() };

        if 0 != unsafe { uname(&mut raw) } {
            return returnerr!("failed to put information about the system in uname");
        }

        let info: Uname = Uname {
            sysname: fromraw(&raw.sysname)?,
            nodename: fromraw(&raw.nodename)?,
            release: fromraw(&raw.release)?,
            version: fromraw(&raw.version)?,
            machine: fromraw(&raw.machine)?,
            domainname: fromraw(&raw._domainname)?,
        };

        Ok(info)
    }
}

impl Display for Uname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{sysname} {version} {release} {machine} {nodename}",
            sysname = self.sysname,
            version = self.version,
            release = self.release,
            machine = self.machine,
            nodename = self.nodename
        )
    }
}

/// The actual function which converts C char arrays into Rust `String`.
fn fromraw(s: &[c_char; 65usize]) -> Result<String> {
    let mut v = s.iter().map(|x| *x as u8).collect::<Vec<u8>>();
    v.retain(|x| *x != 0);
    match String::from_utf8(v) {
        Ok(res) => Ok(res),
        Err(e) => returnerr!(e.to_string()),
    }
}
