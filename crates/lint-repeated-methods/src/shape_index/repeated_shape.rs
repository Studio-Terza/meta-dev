use std::fmt;

use lint_core::DeclaredType;
use lint_core::Fix;

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

impl lint_core::Finding for RepeatedShape {
    fn fix(&self) -> Fix {
        Fix::new("use AsRef<T> for a stored field or From<&T> for a computed value, with a newtype, or a derive")
    }
}
