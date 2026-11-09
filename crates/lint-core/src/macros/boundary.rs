use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ptr;
use std::rc::Rc;

use syn::visit::{self, Visit};
use syn::{File, Macro, Stmt};

use super::Macros;

#[derive(Debug, Default)]
pub struct Expansions {
    pinned: RefCell<Option<HashSet<usize>>>,
    expanded: RefCell<HashMap<usize, Rc<[Stmt]>>>,
}

#[derive(Debug, Default)]
struct Pins(HashSet<usize>);

impl<'ast> Visit<'ast> for Pins {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.0.insert(ptr::from_ref(invocation).addr());
        visit::visit_macro(self, invocation);
    }
}

impl Pins {
    fn in_file(file: &File) -> HashSet<usize> {
        let mut pins = Pins::default();
        pins.visit_file(file);
        pins.0
    }

    fn in_statements(statements: &[Stmt]) -> HashSet<usize> {
        let pinned = statements.iter().fold(Pins::default(), |mut pins, statement| {
            pins.visit_stmt(statement);
            pins
        });
        pinned.0
    }
}

impl Expansions {
    pub fn statements(&self, file: &File, invocation: &Macro) -> Rc<[Stmt]> {
        let address = ptr::from_ref(invocation).addr();
        self.pinned(file, address)
            .map_or_else(|| Rc::from(Macros::statements(invocation)), |()| self.expanded(address, invocation))
    }

    fn pinned(&self, file: &File, address: usize) -> Option<()> {
        let mut pinned = self.pinned.borrow_mut();
        pinned.get_or_insert_with(|| Pins::in_file(file)).contains(&address).then_some(())
    }

    fn expanded(&self, address: usize, invocation: &Macro) -> Rc<[Stmt]> {
        let known = self.expanded.borrow().get(&address).map(Rc::clone);
        known.unwrap_or_else(|| {
            let fresh: Rc<[Stmt]> = Rc::from(Macros::statements(invocation));
            self.pinned.borrow_mut().get_or_insert_with(HashSet::new).extend(Pins::in_statements(&fresh));
            self.expanded.borrow_mut().insert(address, Rc::clone(&fresh));
            fresh
        })
    }
}
