mod impl_target;
mod parameter;
mod struct_fields;

use std::collections::BTreeMap;

use impl_target::ImplTarget;
use parameter::Parameter;
use struct_fields::StructFields;
use syn::visit::Visit;
use syn::{Expr, ExprField, ImplItemFn, ItemImpl, ItemMod, ItemStruct, PathSegment, Stmt};

use super::callee::Callee;
use super::forwarder::Forwarder;
use super::workspace_types::WorkspaceTypes;
use lint_core::Walk;
use lint_core::{Casing, Name};

pub struct Forwarders<'types> {
    types: &'types WorkspaceTypes,
    structs: BTreeMap<Name, StructFields>,
    self_type: Name,
    concrete: Vec<Name>,
    found: Vec<Forwarder>,
}

impl<'types> Forwarders<'types> {
    pub fn new(types: &'types WorkspaceTypes) -> Forwarders<'types> {
        Forwarders {
            types,
            structs: BTreeMap::new(),
            self_type: Name::default(),
            concrete: Vec::new(),
            found: Vec::new(),
        }
    }

    pub fn found(self) -> Vec<Forwarder> {
        self.found
    }

    fn classify(&self, function: &ImplItemFn) -> Option<Forwarder> {
        let params: Vec<Parameter> = function.sig.inputs.iter().filter_map(Parameter::describe).collect();
        let callee = match function.block.stmts.as_slice() {
            [Stmt::Expr(Expr::MethodCall(call), None)]
                if self.receiver_type(&call.receiver, &params).is_some_and(|name| self.types.declares(&name))
                    && call.args.iter().all(|arg| Self::forwards(arg, &params)) =>
            {
                Some(Callee::method(Name::from(&call.method)))
            }
            [Stmt::Expr(Expr::Call(call), None)] if call.args.iter().all(|arg| Self::forwards(arg, &params)) => {
                self.workspace_callee(&call.func)
            }
            _ => None,
        }?;
        Some(Forwarder::new(function.sig.ident.clone(), callee))
    }

    fn receiver_type(&self, expr: &Expr, params: &[Parameter]) -> Option<Name> {
        match expr {
            Expr::Path(path) if path.path.is_ident("self") => Some(self.self_type.clone()),
            Expr::Path(path) => Parameter::named_by(params, &path.path)?.type_name().cloned(),
            Expr::Field(field) => self.field_type(field),
            _ => None,
        }
    }

    fn field_type(&self, field: &ExprField) -> Option<Name> {
        match &*field.base {
            Expr::Path(base) if base.path.is_ident("self") => {
                self.structs.get(&self.self_type)?.field_type(&field.member, &self.concrete)
            }
            _ => None,
        }
    }

    fn workspace_callee(&self, func: &Expr) -> Option<Callee> {
        let segments = Self::path_segments(func)?;
        match segments.as_slice() {
            [.., owner, method] => {
                let owner = if *owner == "Self" { &self.self_type } else { owner };
                (self.types.declares(owner) && method.casing() == Casing::Lower)
                    .then(|| Callee::associated(owner.clone(), method.clone()))
            }
            _ => None,
        }
    }
}

impl Forwarders<'_> {
    fn path_segments(func: &Expr) -> Option<Vec<Name>> {
        match func {
            Expr::Path(path) => Some(path.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect()),
            _ => None,
        }
    }

    fn targeting(&mut self, segment: &PathSegment) {
        self.self_type = Name::from(&segment.ident);
        self.concrete = StructFields::type_arguments(segment);
    }

    fn forwards(expr: &Expr, params: &[Parameter]) -> bool {
        match expr {
            Expr::Path(path) => Parameter::named_by(params, &path.path).is_some(),
            Expr::Field(field) => Self::forwards(&field.base, params),
            Expr::Reference(reference) => Self::forwards(&reference.expr, params),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for Forwarders<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.structs.insert(Name::from(&item.ident), StructFields::describe(item));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        ImplTarget::describe(item).walked_by(self, item);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.found.extend(self.classify(function));
        syn::visit::visit_impl_item_fn(self, function);
    }
}
