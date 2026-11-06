#[derive(Debug, PartialEq, Eq)]
pub struct Kept(u8);
pub struct Held(u8);
impl Clone for Held {
    fn clone(&self) -> Held {
        Held(self.0)
    }
}
impl PartialEq<u8> for Kept {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}
impl From<u8> for Held {
    fn from(value: u8) -> Held {
        Held(value)
    }
}
pub trait Shape {}
impl Shape for Held {}
impl std::fmt::Debug for dyn Shape {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("shape")
    }
}
impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Box<dyn Shape> {
        Box::new(Held(0))
    }
}
pub struct Pair<T>(T);
impl<T: Copy, const N: usize> Clone for Pair<[T; N]> {
    fn clone(&self) -> Pair<[T; N]> {
        Pair(self.0)
    }
}
