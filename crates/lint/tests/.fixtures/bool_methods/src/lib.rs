pub struct Switch;
impl Switch {
    pub fn is_on(&self) -> bool { true }
    pub fn has_power(&self) -> bool { true }
    fn is_off(&self) -> bool { false }
    fn maybe(&self) -> Option<bool> { None }
    fn judge(&self) -> impl Fn(&Switch) -> bool { |_| true }
    fn state(&self) -> State { State::On }
}
pub enum State {
    On,
}
impl PartialEq for Switch {
    fn eq(&self, other: &Switch) -> bool { true }
}
pub trait Probe {
    fn probed(&self) -> bool;
}
fn outer() {
    fn inner() -> bool { true }
}
#[cfg(test)]
mod tests {
    fn gated() -> bool { true }
}
