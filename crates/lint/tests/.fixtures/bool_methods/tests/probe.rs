#[test]
fn probes() {
    assert!(Probing::ready());
}
struct Probing;
impl Probing {
    fn ready() -> bool { true }
}
