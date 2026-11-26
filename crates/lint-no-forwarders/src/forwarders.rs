mod callee;
mod forwarder;
mod impl_target;
mod parameter;
mod struct_fields;

use std::collections::BTreeMap;
use std::path::Path;

use lint_core::Joins;
use impl_target::ImplTarget;
use parameter::Parameter;
use struct_fields::StructFields;
use syntax::{Expr, Ident, ImplItemFn, ItemImpl, ItemMod, ItemStruct, Stmt};

use super::workspace_types::WorkspaceTypes;
use callee::Callee;
use forwarder::{Forwarder, Owned};
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_layout::{Casing, Name};
use lint_sweep::{Contribute, Contributed, Contributing, Contribution, Descend, Facts, Gathers, Kind, Scope};

#[derive(Debug)]
pub struct Forwarders<'file> {
    path: &'file Path,
}

#[derive(Debug)]
pub struct Forwarded<'file> {
    path: &'file Path,
    found: Vec<Forwarder>,
}

#[derive(Clone, Debug)]
pub struct Target {
    self_type: Name,
    self_ident: Option<Name>,
    concrete: Vec<Name>,
}

#[derive(Debug)]
pub struct Candidate {
    ident: Ident,
    params: Vec<Parameter>,
    tail: Expr,
}

#[derive(Debug)]
pub struct Method {
    target: Target,
    candidate: Candidate,
}

#[derive(Debug, Default)]
pub struct Unplaced(Vec<Candidate>);

#[derive(Debug, Default)]
pub struct Gleaned {
    structs: BTreeMap<Name, StructFields>,
    methods: Vec<Method>,
}

struct Classifying<'gleaned> {
    structs: &'gleaned BTreeMap<Name, StructFields>,
    target: &'gleaned Target,
}

impl<'file> Forwarded<'file> {
    pub fn declared(self, types: &WorkspaceTypes) -> Vec<Violation<'file>> {
        let path = self.path;
        let declared = self.found.into_iter().filter(|forwarder| types.declares(forwarder.owner()).is_some());
        declared.map(|forwarder| forwarder.violation(path)).collect()
    }
}

impl Gleaned {
    fn classified(self) -> Vec<Forwarder> {
        let structs = &self.structs;
        self.methods
            .iter()
            .filter_map(|method| Classifying { structs, target: &method.target }.forwarder(&method.candidate))
            .collect()
    }
}

impl Classifying<'_> {
    fn forwarder(&self, candidate: &Candidate) -> Option<Forwarder> {
        self.callee(&candidate.tail, &candidate.params).map(|owned| owned.forwarder(&candidate.ident))
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
                .then_some(&self.target.self_type)
                .or_else(|| Parameter::named_by(params, &path.path).and_then(Parameter::type_name)),
            Expr::Field(field) => Syntax::expr_path(&field.base)
                .filter(|base| base.path.is_ident("self"))
                .and_then(|_| self.structs.get(&self.target.self_type))
                .and_then(|fields| fields.field_type(&field.member, &self.target.concrete)),
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
            .or_else(|| leading.last().and(self.target.self_ident.as_ref()))?;
        (method.casing() == Casing::Lower)
            .then_some(owner)
            .map(|named| Owned::new(Callee::associated(named.clone(), method.clone()), named))
    }
}

impl Candidate {
    fn tail_call(function: &ImplItemFn) -> Option<Candidate> {
        let tail = Some(function.block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Syntax::tail)?;
        Some(Candidate {
            ident: function.sig.ident.clone(),
            params: function.sig.inputs.iter().filter_map(Parameter::describe).collect(),
            tail: tail.clone(),
        })
    }
}

impl Joins for Unplaced {
    fn combine(self, other: Unplaced) -> Unplaced {
        Unplaced(self.0.combine(other.0))
    }
}

impl Joins for Gleaned {
    fn combine(mut self, other: Gleaned) -> Gleaned {
        self.structs.extend(other.structs);
        Gleaned { methods: self.methods.combine(other.methods), ..self }
    }
}


impl Contribute for Forwarders<'_> {
    type Open = Unplaced;
    type Part = Gleaned;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct, Kind::ImplItemFn]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemImpl)] }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Unplaced, Gleaned> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_struct(
        &self,
        item: &ItemStruct,
        facts: &Facts<'_, ItemStruct>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Unplaced, Gleaned> {
        let structs = facts.name().map(|name| (name.clone(), StructFields::describe(item))).into_iter().collect();
        Contributed::new(Unplaced::default(), Gleaned { structs, methods: Vec::new() }, Descend::Over)
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Unplaced, Gleaned> {
        Contributed::descending(ImplTarget::describe(item).target().map_or(Descend::Over, |_| Descend::Into))
    }

    fn off_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        inner: Contribution<Unplaced, Gleaned>,
    ) -> Gleaned {
        inner.concluded(|pending, gleaned| {
            let methods = ImplTarget::describe(item).target().map_or_default(|target| target.placed(pending));
            gleaned.combine(Gleaned { structs: BTreeMap::new(), methods })
        })
    }

    fn on_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Unplaced, Gleaned> {
        Contributed::new(
            Unplaced(Candidate::tail_call(function).into_iter().collect()),
            Gleaned::default(),
            Descend::Into,
        )
    }
}

impl Target {
    fn placed(&self, unplaced: Unplaced) -> Vec<Method> {
        unplaced.0.into_iter().map(|candidate| Method { target: self.clone(), candidate }).collect()
    }
}

impl<'file> Gathers<Forwarded<'file>> for Contributing<Forwarders<'file>> {
    fn gathered(self, file: &SourceFile) -> Forwarded<'file> {
        self.concluded(file, |forwarders, _, gleaned| Forwarded { path: forwarders.path, found: gleaned.classified() })
    }
}

impl<'file> From<&'file Path> for Forwarders<'file> {
    fn from(path: &'file Path) -> Forwarders<'file> {
        Forwarders { path }
    }
}
