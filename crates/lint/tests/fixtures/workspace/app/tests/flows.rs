fn helper() {
    loop {}
}
#[test]
fn checked() {}
#[rstest]
fn cased() {}
#[test_case]
fn listed() {}
proptest! {
    #[test]
    fn property(value in 0..1) {}
}
