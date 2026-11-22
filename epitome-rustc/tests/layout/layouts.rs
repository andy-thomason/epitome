//! Types whose size and alignment `tests/layout_vs_rustc.rs` compares with
//! rustc's. Each `type` here is one comparison.
#![no_std]

pub struct Point { pub x: i32, pub y: i32 }
pub struct Mixed { pub a: u8, pub b: u32, pub c: u16, pub d: u64 }
#[repr(C)]
pub struct MixedC { pub a: u8, pub b: u32, pub c: u16 }
#[repr(C, packed)]
pub struct Packed { pub a: u8, pub b: u32 }
#[repr(align(16))]
pub struct Aligned { pub a: u8 }
pub struct Wrapper(pub u64);
pub struct Empty;

pub enum Unit { A, B, C }
#[repr(u16)]
pub enum Wide { A = 1, B = 300 }
pub enum Neg { A = -1, B = 1 }
pub enum Shape { Circle(u32), Rect { w: u16, h: u16 }, Nothing }
pub enum Big { A(u64), B(u8) }
pub enum Never {}

pub type TPoint = Point;
pub type TMixed = Mixed;
pub type TMixedC = MixedC;
pub type TPacked = Packed;
pub type TAligned = Aligned;
pub type TWrapper = Wrapper;
pub type TEmpty = Empty;
pub type TUnit = Unit;
pub type TWide = Wide;
pub type TNeg = Neg;
pub type TShape = Shape;
pub type TBig = Big;
pub type TNever = Never;

pub type Tuple3 = (u8, u32, u16);
pub type Array = [u16; 3];
pub type ArrayLen = [u8; 2 * 4 + 1];
pub type Str = &'static str;
pub type Slice = &'static [u32];
pub type DynRef = &'static dyn core::fmt::Debug;
pub type FnPtr = fn(u8) -> u8;
pub type Char = char;
pub type U128 = u128;

pub type OptRef = Option<&'static u8>;
pub type OptBool = Option<bool>;
pub type OptChar = Option<char>;
pub type OptU32 = Option<u32>;
pub type OptUnit = Option<Unit>;
pub type OptOptBool = Option<Option<bool>>;
pub type OptSlice = Option<&'static [u8]>;
pub type OptFn = Option<fn()>;
pub type OptNonZero = Option<core::num::NonZero<u32>>;
pub type OptNonNull = Option<core::ptr::NonNull<u8>>;
pub type ResultU64U8 = Result<u64, u8>;
pub type Ordering = core::cmp::Ordering;
pub type OptOrdering = Option<core::cmp::Ordering>;
pub type Range = core::ops::Range<usize>;
