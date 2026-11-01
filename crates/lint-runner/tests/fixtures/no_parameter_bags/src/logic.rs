pub struct Settled {
    depth: u8,
}
impl Settled {
    pub fn deeper(&self) -> u8 {
        self.depth + 1
    }
}
