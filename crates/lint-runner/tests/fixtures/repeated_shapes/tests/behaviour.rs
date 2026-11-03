#[test]
fn a() {
    let items = [1, 2, 3];
    let total: u32 = items.iter().map(|item| item * 2).filter(|item| *item > 3).sum();
    assert_eq!(total, 10);
    assert_eq!(items.len(), 3);
}
#[test]
fn b() {
    let rows = [4, 5, 6];
    let sum: u32 = rows.iter().map(|row| row * 3).filter(|row| *row > 9).sum();
    assert_eq!(sum, 33);
    assert_eq!(rows.len(), 3);
}
#[test]
fn c() {
    g();
}
