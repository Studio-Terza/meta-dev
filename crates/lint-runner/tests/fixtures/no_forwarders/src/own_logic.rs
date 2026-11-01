struct Own;
impl Own {
    fn work(&self) -> u8 { 1 }
}
struct Wrapper { inner: Own }
impl Wrapper {
    fn work(&self) -> u8 {
        self.inner.work() + 1
    }
}
