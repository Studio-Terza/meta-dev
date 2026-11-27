pub struct Switch;
impl Switch {
    pub fn is_on(&self) -> bool { true }
    pub fn has_power(&self) -> bool { true }
    fn is_off(&self) -> bool { false }
}
