mod callee;
mod forwarder;
mod impl_target;
mod parameter;
mod struct_fields;

use std::collections::BTreeMap;
use std::path::Path;

use impl_target::ImplTarget;
use parameter::Parameter;
use struct_fields::StructFields;
use syntax::{Expr, ImplItemFn, ItemImpl, ItemMod, ItemStruct, PathSegment, Stmt};

use super::workspace_types::WorkspaceTypes;
use callee::Callee;
use forwarder::{Forwarder, Owned};
use lint_core::Extended;
use lint_core::Syntax;
use lint_core::Violation;
use lint_layout::{Casing, Name};
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

#[derive(Debug)]
pub struct Forwarders;

#[derive(Debug, Default)]
pub struct Scope {
    structs: BTreeMap<Name, StructFields>,
    self_type: Name,
    self_ident: Option<Name>,
    concrete: Vec<Name>,
    found: Vec<Forwarder>,
}

#[derive(Debug)]
pub struct Forwarded<'file> {
    path: &'file Path,
    found: Vec<Forwarder>,
}

impl<'file> Forwarded<'file> {
    pub fn new(path: &'file Path, scope: Scope) -> Forwarded<'file> {
        Forwarded { path, found: scope.found }
    }

    pub fn declared(self, types: &WorkspaceTypes) -> Vec<Violation<'file>> {
        let path = self.path;
        let declared = self.found.into_iter().filter(|forwarder| types.declares(forwarder.owner()).is_some());
        declared.map(|forwarder| forwarder.violation(path)).collect()
    }
}

impl Scope {
    fn classify(&self, function: &ImplItemFn) -> Option<Forwarder> {
        let params: Vec<Parameter> = function.sig.inputs.iter().filter_map(Parameter::describe).collect();
        Some(function.block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Syntax::tail)
            .and_then(|tail| self.callee(tail, &params))
            .map(|owned| owned.forwarder(&function.sig.ident))
    }

    fn callee(&self, tail: &Expr, params: &[Parameter]) -> Option<Owned> {
        match tail {
            Expr::MethodCall(call) => Parameter::forwarding(&call.args, params)
                .and_then(|()| self.receiver_type(&call.receiver, params))
                .map(|owner| Owned::new(Callee::method(Name::from(&call.method)), owner)),
            Expr::Call(call) => {
                Parameter::forwarding(&call.args, params).and_then(|()| self.workspace_callee(&call.func))
            }
            _ => None,
        }
    }

    fn receiver_type<'name>(&'name self, expr: &Expr, params: &'name [Parameter]) -> Option<&'name Name> {
        match expr {
            Expr::Path(path) => path
                .path
                .is_ident("self")
                .then_some(&self.self_type)
                .or_else(|| Parameter::named_by(params, &path.path).and_then(Parameter::type_name)),
            Expr::Field(field) => Syntax::expr_path(&field.base)
                .filter(|base| base.path.is_ident("self"))
                .and_then(|_| self.structs.get(&self.self_type))
                .and_then(|fields| fields.field_type(&field.member, &self.concrete)),
            _ => None,
        }
    }

    fn workspace_callee(&self, func: &Expr) -> Option<Owned> {
        let segments: Vec<Name> =
            Syntax::expr_path(func)?.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        let (method, leading) = segments.split_last()?;
        let owner = leading
            .last()
            .filter(|owner| AsRef::<str>::as_ref(*owner) != "Self")
            .or_else(|| leading.last().and(self.self_ident.as_ref()))?;
        (method.casing() == Casing::Lower)
            .then_some(owner)
            .map(|named| Owned::new(Callee::associated(named.clone(), method.clone()), named))
    }
}

impl Scope {
    pub fn targeted(self, segment: &PathSegment) -> Scope {
        Scope {
            self_type: Name::from(&segment.ident),
            self_ident: Some(Name::from(&segment.ident)),
            concrete: StructFields::type_arguments(segment),
            ..self
        }
    }
}

impl Fold for Forwarders {
    type Found = Scope;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct, Kind::ItemImpl, Kind::ImplItemFn]
    }

    fn on_item_mod(&self, found: Scope, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Scope> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_struct(&self, found: Scope, item: &ItemStruct, facts: &Facts<'_, ItemStruct>) -> Visit<Scope> {
        let mut structs = found.structs;
        structs.extend(facts.name().map(|name| (name.clone(), StructFields::describe(item))));
        Visit::new(Scope { structs, ..found }, Descend::Over)
    }

    fn on_item_impl(&self, found: Scope, item: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Visit<Scope> {
        ImplTarget::describe(item).walked_by(found)
    }

    fn on_impl_item_fn(&self, found: Scope, function: &ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Visit<Scope> {
        let forwarder = found.classify(function);
        Visit::descending(Scope { found: found.found.extended(forwarder), ..found })
    }
}
