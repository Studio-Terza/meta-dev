use syn::{Fields, FieldsUnnamed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    OneField,
    Other,
}

impl Shape {
    fn of_unnamed(fields: &FieldsUnnamed) -> Self {
        fields.unnamed.first().filter(|_| fields.unnamed.len() == 1).map_or(Self::Other, |_| Self::OneField)
    }
}

impl From<&Fields> for Shape {
    fn from(fields: &Fields) -> Self {
        match fields {
            Fields::Unnamed(unnamed) => Self::of_unnamed(unnamed),
            _ => Self::Other,
        }
    }
}
