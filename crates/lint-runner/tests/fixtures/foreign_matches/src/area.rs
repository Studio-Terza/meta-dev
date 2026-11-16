use crate::shape::Shape;

pub struct Area;

impl Area {
    pub fn measured(shape: Shape) -> Area {
        match shape {
            Shape::Circle(radius) => Area::grown(radius),
            Shape::Square => Area,
        }
    }
}
