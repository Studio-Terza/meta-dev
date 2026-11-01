const fn f(items: &[u8]) {
    let mut rest = items;
    while let [_, tail @ ..] = rest { rest = tail; }
}
fn g() { for _ in 0..3 {} }
