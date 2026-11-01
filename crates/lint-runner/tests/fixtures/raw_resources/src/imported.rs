use std::mem;
fn f(x: u8) {
    mem::forget(x);
}
