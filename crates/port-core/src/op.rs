use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpName(&'static str);

pub trait Op {
    type Port;
    const NAME: OpName;
}
