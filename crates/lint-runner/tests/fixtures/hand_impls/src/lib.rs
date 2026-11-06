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
