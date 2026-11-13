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
use syn::{Expr, ImplItemFn, ItemImpl, ItemMod, ItemStruct, PathSegment, Stmt};

use super::workspace_types::WorkspaceTypes;
use callee::Callee;
use forwarder::Forwarder;
use lint_core::Syntax;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{Casing, Name};
use lint_sweep::{Descend, Finds, Kind, Watch};

#[derive(Debug)]
pub struct Forwarders<'run, 'file> {
    path: &'file Path,
    types: &'run WorkspaceTypes,
    structs: BTreeMap<Name, StructFields>,
    self_type: Name,
    self_ident: Option<Name>,
    concrete: Vec<Name>,
    found: Vec<Violation<'file>>,
}

impl<'run, 'file> Forwarders<'run, 'file> {
    pub fn new(path: &'file Path, types: &'run WorkspaceTypes) -> Forwarders<'run, 'file> {
        Forwarders {
            path,
            types,
            structs: BTreeMap::new(),
            self_type: Name::default(),
            self_ident: None,
            concrete: Vec::new(),
            found: Vec::new(),
        }
    }

    fn classify(&self, function: &ImplItemFn) -> Option<Forwarder> {
        let params: Vec<Parameter> = function.sig.inputs.iter().filter_map(Parameter::describe).collect();
        Some(function.block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Syntax::tail)
            .and_then(|tail| self.callee(tail, &params))
            .map(|callee| Forwarder::new(&function.sig.ident, callee))
    }

    fn callee(&self, tail: &Expr, params: &[Parameter]) -> Option<Callee> {
        match tail {
            Expr::MethodCall(call) => Parameter::forwarding(&call.args, params)
                .and_then(|()| self.receiver_type(&call.receiver, params))
                .filter(|name| self.types.declares(name).is_some())
                .map(|_| Callee::method(Name::from(&call.method))),
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

    fn workspace_callee(&self, func: &Expr) -> Option<Callee> {
        let segments: Vec<Name> =
            Syntax::expr_path(func)?.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        let (method, leading) = segments.split_last()?;
        let owner = leading
            .last()
            .filter(|owner| AsRef::<str>::as_ref(*owner) != "Self")
            .or_else(|| leading.last().and(self.self_ident.as_ref()))?;
        self.types
            .declares(owner)
            .filter(|_| method.casing() == Casing::Lower)
            .map(|_| Callee::associated(owner.clone(), method.clone()))
    }
}

impl Forwarders<'_, '_> {
    pub fn targeted(&mut self, segment: &PathSegment) -> Descend {
        self.self_type = Name::from(&segment.ident);
        self.self_ident = Some(Name::from(&segment.ident));
        self.concrete = StructFields::type_arguments(segment);
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Forwarders<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct, Kind::ItemImpl, Kind::ImplItemFn]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        self.structs.insert(Name::from(&item.ident), StructFields::describe(item));
        Descend::Over
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        ImplTarget::describe(item).walked_by(self)
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        let path = self.path;
        self.found.extend(self.classify(function).map(|forwarder| forwarder.violation(path)));
        Descend::Into
    }
}

impl<'file> Finds<'file> for Forwarders<'_, 'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.found
    }
}
