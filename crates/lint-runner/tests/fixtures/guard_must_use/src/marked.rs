#[must_use]
pub struct Held;
impl Drop for Held {
    fn drop(&mut self) {}
}
