mod impl_target;
mod parameter;
mod struct_fields;

use std::collections::BTreeMap;

use impl_target::ImplTarget;
use parameter::Parameter;
use struct_fields::StructFields;
use syn::visit::Visit;
use syn::{Expr, ExprField, Ident, ImplItemFn, ItemImpl, ItemMod, ItemStruct, PathSegment, Stmt};

use super::callee::Callee;
use super::forwarder::Forwarder;
use super::workspace_types::WorkspaceTypes;
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
        let callee = match function.block.stmts.as_slice() {
            [Stmt::Expr(Expr::MethodCall(call), None)]
                if self
                    .receiver_type(&call.receiver, &params)
                    .is_some_and(|name| self.types.declares(name).is_some())
                    && call.args.iter().all(|arg| Self::forwarded(arg, &params).is_some()) =>
            {
                Some(Callee::method(&call.method))
            }
            [Stmt::Expr(Expr::Call(call), None)]
                if call.args.iter().all(|arg| Self::forwarded(arg, &params).is_some()) =>
            {
                self.workspace_callee(&call.func)
            }
            _ => None,
        }?;
        Some(Forwarder::new(&function.sig.ident, callee))
    }

    fn receiver_type<'name>(&'name self, expr: &'file Expr, params: &'name [Parameter]) -> Option<&'name Name> {
        match expr {
            Expr::Path(path) if path.path.is_ident("self") => Some(&self.self_type),
            Expr::Path(path) => Parameter::named_by(params, &path.path)?.type_name(),
            Expr::Field(field) => self.field_type(field),
            _ => None,
        }
    }

    fn field_type(&self, field: &'file ExprField) -> Option<&Name> {
        match &*field.base {
            Expr::Path(base) if base.path.is_ident("self") => {
                self.structs.get(&self.self_type)?.field_type(&field.member, &self.concrete)
            }
            _ => None,
        }
    }

    fn workspace_callee(&self, func: &'file Expr) -> Option<Callee<'file>> {
        let segments = Self::path_segments(func)?;
        match segments.as_slice() {
            [.., owner, method] => {
                let owner = if *owner == "Self" { self.self_ident? } else { owner };
                (self.types.declares(&Name::from(owner)).is_some() && Name::from(*method).casing() == Casing::Lower)
                    .then(|| Callee::associated(owner, method))
            }
            _ => None,
        }
    }
}

impl<'file> Forwarders<'_, 'file> {
    fn path_segments(func: &'file Expr) -> Option<Vec<&'file Ident>> {
        match func {
            Expr::Path(path) => Some(path.path.segments.iter().map(|segment| &segment.ident).collect()),
            _ => None,
        }
    }

    fn targeting(&mut self, segment: &'file PathSegment) {
        self.self_type = Name::from(&segment.ident);
        self.self_ident = Some(&segment.ident);
        self.concrete = StructFields::type_arguments(segment);
    }

    fn forwarded<'expr>(expr: &'expr Expr, params: &[Parameter]) -> Option<&'expr Expr> {
        match expr {
            Expr::Path(path) => Parameter::named_by(params, &path.path).map(|_| expr),
            Expr::Field(field) => Self::forwarded(&field.base, params),
            Expr::Reference(reference) => Self::forwarded(&reference.expr, params),
            _ => None,
        }
    }
}

impl<'file> Visit<'file> for Forwarders<'_, 'file> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
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
