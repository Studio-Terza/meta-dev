fn area(shape: Shape) -> u8 {
    match shape {
        Shape::Circle(radius) => grow(radius),
        Shape::Square => 1,
    }
}
