struct Sized { inner: Vec<u8> }
impl Sized {
    fn size(&self) -> usize {
        self.inner.len()
    }
}
