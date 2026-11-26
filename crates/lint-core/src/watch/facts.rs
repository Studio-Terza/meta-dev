mod boundary;

use std::cell::OnceCell;

use syntax::{Attribute, ItemImpl};

use lint_layout::{Name, TestMarks};

use super::boundary::Address;
use crate::syntax::Syntax;
use crate::walk::Walk;
use boundary::{Attributed, Identified};

#[derive(Debug)]
pub struct Facts<'node, Node> {
    node: &'node Node,
    address: Address,
    gate: OnceCell<Walk>,
    name: OnceCell<Option<Name>>,
    marked: OnceCell<Option<&'node Node>>,
}

impl<'node, Node> Facts<'node, Node> {
    #[must_use]
    pub fn new(node: &'node Node) -> Facts<'node, Node> {
        Facts {
            node,
            address: Address::from(node),
            gate: OnceCell::new(),
            name: OnceCell::new(),
            marked: OnceCell::new(),
        }
    }

    #[must_use]
    pub fn address(&self) -> Address {
        self.address
    }
}

impl<'node, Node: Attributed> Facts<'node, Node> {
    pub fn gate(&self) -> Walk {
        *self.gate.get_or_init(|| Walk::outside_tests(self.node.attributes()))
    }

    pub fn test_mark(&self) -> Option<&'node Attribute> {
        TestMarks::mark_test(self.node.attributes())
    }
}

impl<Node: Identified> Facts<'_, Node> {
    pub fn name(&self) -> Option<&Name> {
        self.name.get_or_init(|| self.node.identity()).as_ref()
    }
}

impl<'node> Facts<'node, ItemImpl> {
    #[must_use]
    pub fn inherent(&self) -> Option<&'node ItemImpl> {
        *self.marked.get_or_init(|| Syntax::inherent(self.node))
    }
}
