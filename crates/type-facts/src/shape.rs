use syn::{Fields, FieldsUnnamed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    OneField,
    Other,
}

impl Shape {
    fn of_unnamed(fields: &FieldsUnnamed) -> Shape {
        fields.unnamed.first().filter(|_| fields.unnamed.len() == 1).map_or(Shape::Other, |_| Shape::OneField)
    }
}

impl From<&Fields> for Shape {
    fn from(fields: &Fields) -> Shape {
        match fields {
            Fields::Unnamed(unnamed) => Shape::of_unnamed(unnamed),
            _ => Shape::Other,
        }
    }
}
