pub struct Port;
impl Port {
    pub fn open(&self, name: &str) -> String {
        name.to_owned()
    }
    fn sizes(&self, sizes: Option<Vec<u8>>, scale: &[f64]) {}
    fn judged(&self, judge: impl Fn(&Port) -> bool) {}
    fn bounded<Text: AsRef<str>>(&self, text: Text) {}
    fn meant(&self, port: Port) -> Port {
        port
    }
    fn unit(&self) {}
}
pub trait Named {
    type Spelled = String;
    fn name(&self) -> char;
}
impl Iterator for Port {
    type Item = u8;
    fn next(&mut self) -> Option<Port> {
        None
    }
}
pub fn counted(items: std::vec::Vec<i64>) -> usize {
    items.len()
}
