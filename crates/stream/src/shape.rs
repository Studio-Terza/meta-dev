use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

use crate::form::{Form, Forms};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case")]
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
