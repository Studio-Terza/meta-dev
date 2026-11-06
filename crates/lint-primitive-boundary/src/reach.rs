mod macro_body;

use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ItemMod, Macro};

use macro_body::MacroBody;

use lint_core::Gate;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    Blocking,
    Widened,
}

impl Reach {
    pub fn of_file(file: &SourceFile, support: &[PathBuf]) -> Reach {
        match support.iter().find(|path| *path == file.path()) {
            Some(_) => Reach::Widened,
            None => Self::of_role(file),
        }
    }

    fn of_role(file: &SourceFile) -> Reach {
        match file.shipped() {
            Some(_) => Reach::Blocking,
            None => Reach::Widened,
        }
    }

    fn inside(self, module: &ItemMod) -> Option<Reach> {
        match Gate::BOUNDARY.names(module) {
            Some(_) => None,
            None => Some(self.within(Walk::outside_tests(&module.attrs))),
        }
    }

    fn within(self, walk: Walk) -> Reach {
        match walk {
            Walk::Entered => self,
            Walk::Skipped => Reach::Widened,
        }
    }
}

pub trait Reaching: Sized {
    fn reach(&mut self) -> &mut Reach;

    fn reached(&mut self, reach: Reach, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(self.reach(), reach);
        visit(self);
        *self.reach() = outer;
    }

    fn widened(&mut self, visit: impl FnOnce(&mut Self)) {
        self.reached(Reach::Widened, visit);
    }

    fn entered(&mut self, module: &ItemMod, mut visit: impl FnMut(&mut Self)) {
        self.reach().inside(module).into_iter().for_each(|reach| self.reached(reach, &mut visit));
    }

    fn expanded(&mut self, invocation: &Macro)
    where
        Self: for<'ast> Visit<'ast>,
    {
        let body = MacroBody::read(invocation);
        self.widened(|visitor| body.walked(visitor));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reached<'file> {
    reach: Reach,
    violation: Violation<'file>,
}

impl<'file> Reached<'file> {
    pub fn new(reach: Reach, violation: Violation<'file>) -> Reached<'file> {
        Reached { reach, violation }
    }

    pub fn reach(&self) -> Reach {
        self.reach
    }

    pub fn into_violation(self) -> Violation<'file> {
        self.violation
    }
}
