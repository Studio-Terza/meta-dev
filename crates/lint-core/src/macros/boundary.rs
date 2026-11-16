mod macro_calls;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use syntax::{Attribute, Item, Macro, MacroTree, Stmt};

use super::Macros;
use crate::watch::Address;
use macro_calls::MacroCalls;

#[derive(Debug, Default)]
struct Memo<Facts>(RefCell<HashMap<Address, Facts>>);

impl<Facts: Clone> Memo<Facts> {
    fn read(&self, pinned: Option<Address>, made: impl FnOnce() -> Facts) -> Facts {
        let known = pinned.and_then(|at| self.0.borrow().get(&at).cloned());
        known.unwrap_or_else(|| {
            let fresh = made();
            self.0.borrow_mut().extend(pinned.map(|at| (at, fresh.clone())));
            fresh
        })
    }
}

#[derive(Debug, Default)]
pub struct Expansions {
    pinned: RefCell<HashSet<Address>>,
    statements: Memo<Rc<[Stmt]>>,
    calls: Memo<Rc<[Macro]>>,
    attributes: Memo<Rc<[Attribute]>>,
    items: Memo<Rc<[Item]>>,
}

impl Expansions {
    pub fn pin(&self, invocation: &Macro) {
        self.pinned.borrow_mut().insert(Address::from(invocation));
    }

    pub fn statements(&self, invocation: &Macro) -> Rc<[Stmt]> {
        self.statements.read(self.held(Address::from(invocation)), || Rc::from(Macros::statements(invocation)))
    }

    pub fn calls(&self, at: Address, invocation: &Macro) -> Rc<[Macro]> {
        let pinned = self.held(at);
        let calls = self.calls.read(pinned, || Rc::from(MacroCalls::within(invocation)));
        let inner = pinned.map(|_| calls.iter().map(Address::from));
        self.pinned.borrow_mut().extend(inner.into_iter().flatten());
        calls
    }

    pub fn items(&self, invocation: &Macro) -> Rc<[Item]> {
        self.items.read(self.held(Address::from(invocation)), || Rc::from(MacroTree::items(invocation)))
    }

    pub fn attributes(&self, invocation: &Macro) -> Rc<[Attribute]> {
        self.attributes.read(self.held(Address::from(invocation)), || Rc::from(Macros::attributes(invocation)))
    }

    fn held(&self, at: Address) -> Option<Address> {
        self.pinned.borrow().get(&at).copied()
    }
}
