use named_derive::Named;
use strum::VariantArray;

use crate::form::Form;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    #[name("sentence")]
    Sentence,
    #[name("item")]
    Item,
    #[name("bullet")]
    Bullet,
    #[name("quote")]
    Quote,
}

impl From<Form> for Shape {
    fn from(form: Form) -> Shape {
        match form {
            Form::Sentence => Shape::Sentence,
            Form::Item(..) => Shape::Item,
            Form::Bullet => Shape::Bullet,
            Form::Quote => Shape::Quote,
        }
    }
}
