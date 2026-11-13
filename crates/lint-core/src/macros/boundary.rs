use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ptr;
use std::rc::Rc;

use syn::{Macro, Stmt};

use super::Macros;

#[derive(Debug, Default)]
pub struct Expansions {
    pinned: RefCell<HashSet<usize>>,
    expanded: RefCell<HashMap<usize, Rc<[Stmt]>>>,
}

impl Expansions {
    pub fn pin(&self, invocation: &Macro) {
        self.pinned.borrow_mut().insert(ptr::from_ref(invocation).addr());
    }

    pub fn statements(&self, invocation: &Macro) -> Rc<[Stmt]> {
        let address = ptr::from_ref(invocation).addr();
        let pinned = self.pinned.borrow().contains(&address).then_some(address);
        pinned.map_or_else(|| Rc::from(Macros::statements(invocation)), |held| self.expanded(held, invocation))
    }

    fn expanded(&self, address: usize, invocation: &Macro) -> Rc<[Stmt]> {
        let known = self.expanded.borrow().get(&address).map(Rc::clone);
        known.unwrap_or_else(|| {
            let fresh: Rc<[Stmt]> = Rc::from(Macros::statements(invocation));
            self.expanded.borrow_mut().insert(address, Rc::clone(&fresh));
            fresh
        })
    }
}
