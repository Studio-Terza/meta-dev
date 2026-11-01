use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub struct Contents<'text>(&'text str);
