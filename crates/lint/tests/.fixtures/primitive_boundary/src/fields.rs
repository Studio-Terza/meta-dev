pub struct Label(String);
pub struct Pair(u32, u32);
pub struct Record {
    count: usize,
    label: Label,
}
pub enum Shape {
    Named(String),
    Sized { width: i32 },
    Empty,
}
pub struct Wrapped<Inner = u16> {
    inner: Inner,
}
pub type Size = u64;
pub union Bits {
    whole: u128,
}
