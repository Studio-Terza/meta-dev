mod boundary;
pub struct Held(String);
impl Held {
    pub fn copied(&self) -> String {
        self.0.clone()
    }
}
