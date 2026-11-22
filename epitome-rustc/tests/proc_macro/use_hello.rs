// Uses the proc macros of `hello.rs`: a derive, a function-like macro and
// an attribute macro.

pub trait Hello {
    fn hello() -> &'static str;
}

/// A documented struct.
#[derive(hello::Hello, Clone)]
pub struct Point {
    pub x: i32,
    #[cfg(windows)]
    pub windows_only: u8,
}

#[derive(hello::Hello)]
pub enum Shape {
    Circle,
}

pub fn echoed() -> u32 {
    hello::echo!(1 + 2) * 3
}

#[hello::twice]
pub fn once() -> u32 {
    1
}

pub const SHOWN: &str = hello::show!(a => b'x' + r#"q"#);
