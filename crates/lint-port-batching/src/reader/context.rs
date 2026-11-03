use std::collections::BTreeSet;

use syn::{FnArg, Signature, Type};

use lint_layout::Name;
use lint_paths::Module;
use lint_placement::CrateName;

use super::scope::Scope;
use crate::holders::Holders;
use crate::judge::Judge;
use crate::spelling::Spelling;

#[derive(Debug)]
pub struct Context<'run> {
    judge: &'run Judge<'run>,
    holders: &'run Holders,
    crate_name: CrateName,
}

impl<'run> Context<'run> {
    pub fn new(judge: &'run Judge<'run>, holders: &'run Holders, crate_name: CrateName) -> Context<'run> {
        Context { judge, holders, crate_name }
    }

    pub fn judge(&self) -> &Judge<'run> {
        self.judge
    }

    pub fn held(&self, owner: &Type) -> Option<&'run BTreeSet<Name>> {
        let holders = self.holders;
        Spelling::last(owner).and_then(|owner| holders.fields(&self.crate_name, &owner))
    }

    pub fn bindings(&self, module: &Module<'_>, generics: &BTreeSet<Name>, signature: &Signature) -> BTreeSet<Name> {
        signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => Some(typed),
                FnArg::Receiver(_) => None,
            })
            .filter(|typed| self.parameter(module, generics, &typed.ty).is_some())
            .filter_map(|typed| Scope::bound(&typed.pat))
            .collect()
    }

    fn parameter(&self, module: &Module<'_>, generics: &BTreeSet<Name>, ty: &Type) -> Option<()> {
        self.judge.port_type(module, generics, ty).map(|_| ()).or_else(|| self.holding(ty).map(|_| ()))
    }

    fn holding(&self, ty: &Type) -> Option<&Name> {
        Spelling::last(ty).and_then(|owner| self.holders.holder(&self.crate_name, &owner))
    }
}
