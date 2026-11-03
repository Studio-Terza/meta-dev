pub trait Shape {
    fn area(&self);
}

pub struct Square;

impl Shape for Square {
    fn area(&self) {}
}
