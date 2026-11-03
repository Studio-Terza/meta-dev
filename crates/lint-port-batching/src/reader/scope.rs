use std::collections::BTreeSet;

use syn::{ExprClosure, ExprField, ExprPath, Member, Pat};

use lint_layout::Name;

use super::finding::{Port, Within};

#[derive(Debug, Default)]
pub struct Scope<'run> {
    generics: BTreeSet<Name>,
    held: Option<&'run BTreeSet<Name>>,
    bindings: BTreeSet<Name>,
    within: Option<Within>,
}

impl<'run> Scope<'run> {
    const SELF: &str = "self";

    pub fn declared(generics: BTreeSet<Name>, held: Option<&'run BTreeSet<Name>>) -> Scope<'run> {
        Scope { generics, held, ..Scope::default() }
    }

    pub fn generics(&self) -> &BTreeSet<Name> {
        &self.generics
    }

    pub fn within(&self) -> Option<Within> {
        self.within
    }

    pub fn function(&self, generics: BTreeSet<Name>, bindings: BTreeSet<Name>) -> Scope<'run> {
        Scope { generics, held: self.held, bindings, within: None }
    }

    pub fn replace_within(&mut self, within: Option<Within>) -> Option<Within> {
        std::mem::replace(&mut self.within, within)
    }

    pub fn shadowed(&mut self, closure: &ExprClosure) -> BTreeSet<Name> {
        let shadowed: BTreeSet<Name> = closure.inputs.iter().filter_map(Self::bound).collect();
        let kept = self.bindings.iter().filter(|binding| !shadowed.contains(*binding)).cloned().collect();
        std::mem::replace(&mut self.bindings, kept)
    }

    pub fn unshadowed(&mut self, bindings: BTreeSet<Name>) {
        self.bindings = bindings;
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
            .filter(|member| self.held.is_some_and(|held| held.contains(member)))
            .map(Port::Field)
    }
}
