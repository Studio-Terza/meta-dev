fn f() {
    for _ in [1, 2] {}
    while true {}
    loop {}
}
fn g() {
    for _ in [1] {
        if h() {
            break;
        } else {
            continue;
        }
    }
}
