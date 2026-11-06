use std::fmt;

use crate::Form;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    Sentence,
    Item,
    Bullet,
    Quote,
}

impl From<Form> for Shape {
    fn from(form: Form) -> Shape {
        match form {
            Form::Sentence => Shape::Sentence,
            Form::Item(_) => Shape::Item,
            Form::Bullet => Shape::Bullet,
            Form::Quote => Shape::Quote,
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Shape::Sentence => "sentence",
            Shape::Item => "item",
            Shape::Bullet => "bullet",
            Shape::Quote => "quote",
        })
    }
}
