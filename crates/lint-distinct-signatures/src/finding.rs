use std::fmt;

use lint_core::Name;

use super::shape::Shape;

pub struct Finding {
    names: Vec<Name>,
    shape: Shape,
}

impl Finding {
    pub fn new(names: Vec<Name>, shape: Shape) -> Finding {
        Finding { names, shape }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.names.iter().map(Name::to_string).collect();
        write!(
            formatter,
            "{} share the signature {}; distinguish the types they take or return",
            names.join(", "),
            self.shape
        )
    }
}
