use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

use super::shape::Shape;

pub struct Finding<'index> {
    names: &'index [Name],
    shape: Shape,
}

impl<'index> Finding<'index> {
    pub fn new(names: &'index [Name], shape: Shape) -> Finding<'index> {
        Finding { names, shape }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.names.iter().map(Name::to_string).collect();
        write!(formatter, "{} share the signature {}", names.join(", "), self.shape)
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new("distinguish the types they take or return")
    }
}
