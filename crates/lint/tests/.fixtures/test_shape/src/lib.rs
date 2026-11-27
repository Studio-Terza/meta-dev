pub struct Shipped;
impl Shipped {
    pub fn shipped(&self) -> u8 {
        1
    }
}
#[cfg(test)]
mod tests {
    struct Fake;
    fn helper() {}
    #[test]
    fn gated() {
        assert_eq!(1, 1);
    }
}
