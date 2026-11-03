use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Text<'text>(&'text str);
