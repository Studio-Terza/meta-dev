use std::fmt;

use lint_core::DeclaredType;

use super::method_shape::MethodShape;

#[derive(Debug)]
pub struct RepeatedShape {
    shape: MethodShape,
    types: Vec<DeclaredType>,
}

impl RepeatedShape {
    pub fn new(shape: MethodShape, types: Vec<DeclaredType>) -> RepeatedShape {
        RepeatedShape { shape, types }
    }
}

impl fmt::Display for RepeatedShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let types: Vec<String> = self.types.iter().map(ToString::to_string).collect();
        write!(formatter, "{} on {}", self.shape, types.join(", "))
    }
}
