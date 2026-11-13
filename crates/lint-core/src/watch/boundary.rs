use std::ptr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address(usize);

impl<Node> From<&Node> for Address {
    fn from(node: &Node) -> Address {
        Address(ptr::from_ref(node).addr())
    }
}
