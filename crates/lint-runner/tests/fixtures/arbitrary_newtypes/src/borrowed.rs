use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub struct Title(&'static str);
