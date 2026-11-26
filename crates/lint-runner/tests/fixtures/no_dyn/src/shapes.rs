pub trait Shape {}

pub struct Shapes {
    pub held: Vec<Box<dyn Shape>>,
}

pub const NAMED: &str = "dyn Shape";
