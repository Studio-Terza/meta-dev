use std::fmt;

use crate::form::Form;

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
        match self {
            Shape::Sentence => formatter.write_str("sentence"),
            Shape::Item => formatter.write_str("item"),
            Shape::Bullet => formatter.write_str("bullet"),
            Shape::Quote => formatter.write_str("quote"),
        }
    }
}
