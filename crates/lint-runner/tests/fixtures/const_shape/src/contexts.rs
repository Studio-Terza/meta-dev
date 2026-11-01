pub static X: u32 = 1u8 as u32;
pub static Y: u32 = { let mut i = 0; while i < 3 { i += 1; } i };
pub trait T { const X: u32 = 1u8 as u32; }
pub struct A { pub b: [u8; N as usize] }
enum E { A = 1u8 as isize }
