#![no_std]
// The built-ins expansion does itself.

pub const NAME: &str = stringify!(a + b * c);
pub const JOINED: &str = concat!("x", 1, 'y', true, -2);
pub const LINE: u32 = line!();

cfg_select! {
    unix => {
        pub fn os() -> &'static str { "unix" }
    }
    _ => {
        pub fn os() -> &'static str { "other" }
    }
}

pub fn is_unix() -> bool {
    cfg!(unix) && !cfg!(windows)
}

macro_rules! counted {
    ($($x:ident),*) => { [$(stringify!($x)),*] };
}

pub fn names() -> [&'static str; 3] {
    counted!(alpha, beta, gamma)
}

#[cfg_attr(unix, inline)]
pub fn attr() {}

#[cfg(windows)]
pub fn gone() {}
