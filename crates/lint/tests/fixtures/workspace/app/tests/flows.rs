fn helper() {
    loop {}
}
#[test]
fn checked() {}
proptest! {
    #[test]
    fn property(value in 0..1) {}
}
