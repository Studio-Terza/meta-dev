use std::collections::BTreeSet;

use syn::{ExprClosure, ExprField, ExprPath, Member, Pat};

use lint_layout::Name;

use super::finding::{Port, Within};

#[derive(Clone, Debug, Default)]
pub struct Scope {
    generics: BTreeSet<Name>,
    held: BTreeSet<Name>,
    bindings: BTreeSet<Name>,
    within: Option<Within>,
}

impl Scope {
    const SELF: &str = "self";

    pub fn declared(generics: BTreeSet<Name>, held: BTreeSet<Name>) -> Scope {
        Scope { generics, held, ..Scope::default() }
    }

    pub fn generics(&self) -> &BTreeSet<Name> {
        &self.generics
    }

    pub fn within(&self) -> Option<&Within> {
        self.within.as_ref()
    }

    pub fn function(&self, generics: BTreeSet<Name>, bindings: BTreeSet<Name>) -> Scope {
        Scope { generics, held: self.held.clone(), bindings, within: None }
    }

    pub fn entered(&self, within: Option<Within>) -> Scope {
        Scope { within: self.within.clone().or(within), ..self.clone() }
    }

    pub fn paced(&self, paced: Option<&Name>) -> Scope {
        Scope { within: self.within.clone().filter(|_| paced.is_none()), ..self.clone() }
    }

    pub fn shadowed(&self, closure: &ExprClosure) -> Scope {
        let shadowed: BTreeSet<Name> = closure.inputs.iter().filter_map(Self::bound).collect();
        Scope { bindings: self.bindings.difference(&shadowed).cloned().collect(), ..self.clone() }
    }

    pub fn bound(pattern: &Pat) -> Option<Name> {
        match pattern {
            Pat::Ident(binding) => Some(Name::from(&binding.ident)),
            Pat::Type(typed) => Self::bound(&typed.pat),
            _ => None,
        }
    }

    pub fn reached_path(&self, path: &ExprPath) -> Option<Port> {
        path.path.get_ident().map(Name::from).filter(|name| self.bindings.contains(name)).map(Port::Binding)
    }

    pub fn reached_field(&self, field: &ExprField) -> Option<Port> {
        let member = match &field.member {
            Member::Named(member) => Some(Name::from(member)),
            Member::Unnamed(_) => None,
        };
        member
            .filter(|_| matches!(field.base.as_ref(), syn::Expr::Path(base) if base.path.is_ident(Self::SELF)))
            .filter(|member| self.held.contains(member))
            .map(Port::Field)
    }
}
