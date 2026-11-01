struct Inner;
impl Inner {
    fn work(&self) -> u8 { 1 }
}
struct Outer { inner: Inner }
impl Outer {
    fn work(&self) -> u8 {
        self.inner.work()
    }
}
