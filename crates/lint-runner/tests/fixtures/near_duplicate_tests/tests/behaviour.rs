#[test]
fn a() {
    assert_eq!(f(1, "x"), true);
}
#[test]
fn b() {
    assert_eq!(f(2, "y"), false);
}
#[test]
fn c() {
    g();
}
