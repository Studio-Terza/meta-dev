mod impl_target;
mod parameter;
mod struct_fields;

use std::collections::BTreeMap;

use impl_target::ImplTarget;
use parameter::Parameter;
use struct_fields::StructFields;
use syn::visit::Visit;
use syn::{Expr, Ident, ImplItemFn, ItemImpl, ItemMod, ItemStruct, PathSegment, Stmt};

use super::callee::Callee;
use super::forwarder::Forwarder;
use super::workspace_types::WorkspaceTypes;
use lint_core::Syntax;
use lint_core::Walk;
use lint_layout::{Casing, Name};

pub struct Forwarders<'types, 'file> {
    types: &'types WorkspaceTypes,
    structs: BTreeMap<Name, StructFields<'file>>,
    self_type: Name,
    self_ident: Option<&'file Ident>,
    concrete: Vec<Name>,
    found: Vec<Forwarder<'file>>,
}

impl<'types, 'file> Forwarders<'types, 'file> {
    pub fn new(types: &'types WorkspaceTypes) -> Forwarders<'types, 'file> {
        Forwarders {
            types,
            structs: BTreeMap::new(),
            self_type: Name::default(),
            self_ident: None,
            concrete: Vec::new(),
            found: Vec::new(),
        }
    }

    pub fn found(self) -> Vec<Forwarder<'file>> {
        self.found
    }

    fn classify(&self, function: &'file ImplItemFn) -> Option<Forwarder<'file>> {
        let params: Vec<Parameter> = function.sig.inputs.iter().filter_map(Parameter::describe).collect();
        Some(function.block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Syntax::tail)
            .and_then(|tail| self.callee(tail, &params))
            .map(|callee| Forwarder::new(&function.sig.ident, callee))
    }

    fn callee(&self, tail: &'file Expr, params: &[Parameter]) -> Option<Callee<'file>> {
        match tail {
            Expr::MethodCall(call) => Parameter::forwarding(&call.args, params)
                .and_then(|()| self.receiver_type(&call.receiver, params))
                .filter(|name| self.types.declares(name).is_some())
                .map(|_| Callee::method(&call.method)),
            Expr::Call(call) => {
                Parameter::forwarding(&call.args, params).and_then(|()| self.workspace_callee(&call.func))
            }
            _ => None,
        }
    }

    fn receiver_type<'name>(&'name self, expr: &'file Expr, params: &'name [Parameter]) -> Option<&'name Name> {
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

    fn workspace_callee(&self, func: &'file Expr) -> Option<Callee<'file>> {
        let segments: Vec<&Ident> =
            Syntax::expr_path(func)?.path.segments.iter().map(|segment| &segment.ident).collect();
        let (method, leading) = segments.split_last()?;
        let owner =
            leading.last().copied().filter(|owner| *owner != "Self").or_else(|| leading.last().and(self.self_ident))?;
        self.types
            .declares(&Name::from(owner))
            .filter(|_| Name::from(*method).casing() == Casing::Lower)
            .map(|_| Callee::associated(owner, method))
    }
}

impl<'file> Forwarders<'_, 'file> {
    pub fn targeted(&mut self, segment: &'file PathSegment, item: &'file ItemImpl) {
        self.self_type = Name::from(&segment.ident);
        self.self_ident = Some(&segment.ident);
        self.concrete = StructFields::type_arguments(segment);
        syn::visit::visit_item_impl(self, item);
    }
}

impl<'file> Visit<'file> for Forwarders<'_, 'file> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'file ItemStruct) {
        self.structs.insert(Name::from(&item.ident), StructFields::describe(item));
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        ImplTarget::describe(item).walked_by(self, item);
    }

    fn visit_impl_item_fn(&mut self, function: &'file ImplItemFn) {
        self.found.extend(self.classify(function));
        syn::visit::visit_impl_item_fn(self, function);
    }
}
