mod boundary;

use std::collections::BTreeMap;

use syn::{Expr, ItemConst, Local};

use lint_core::Syntax;
use lint_layout::Name;

pub use boundary::{BodyId, Scopes};

#[derive(Debug)]
pub struct Bound {
    name: Name,
    value: Expr,
}

impl Bound {
    pub fn constant(item: &ItemConst) -> Bound {
        Bound { name: Name::from(&item.ident), value: Expr::clone(&item.expr) }
    }

    pub fn local(local: &Local) -> Option<Bound> {
        Syntax::pat_ident(&local.pat)
            .zip(local.init.as_ref())
            .map(|(name, init)| Bound { name: Name::from(&name.ident), value: Expr::clone(&init.expr) })
    }
}

#[derive(Debug, Default)]
pub struct Bindings<'run> {
    bound: BTreeMap<&'run Name, &'run Expr>,
}

impl<'run> FromIterator<&'run Bound> for Bindings<'run> {
    fn from_iter<Bounds: IntoIterator<Item = &'run Bound>>(bounds: Bounds) -> Bindings<'run> {
        Bindings { bound: bounds.into_iter().map(|bound| (&bound.name, &bound.value)).collect() }
    }
}

impl<'run> Bindings<'run> {
    pub fn bound(&self, name: &Name) -> Option<&'run Expr> {
        self.bound.get(name).copied()
    }
}
