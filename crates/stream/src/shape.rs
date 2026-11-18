use named_derive::Named;

use crate::form::{Form, Forms};

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    Sentence,
    Item,
    Bullet,
    Quote,
}

impl From<Form> for Shape {
    fn from(form: Form) -> Shape {
        form.formed()
    }
}

impl Forms for Shape {
    const SENTENCE: Shape = Shape::Sentence;
    const ITEM: Shape = Shape::Item;
    const BULLET: Shape = Shape::Bullet;
    const QUOTE: Shape = Shape::Quote;
}
