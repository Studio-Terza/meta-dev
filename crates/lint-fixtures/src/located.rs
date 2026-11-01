use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Located(&'static str);
