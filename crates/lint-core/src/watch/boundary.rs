use std::ptr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(usize);

impl<Node> From<&Node> for Address {
    fn from(node: &Node) -> Address {
        Address(ptr::from_ref(node).addr())
    }
}
