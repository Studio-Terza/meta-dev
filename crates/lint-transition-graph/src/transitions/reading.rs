mod endpoints;
mod peeling;

use std::collections::BTreeMap;
use std::path::Path;

use graph::{
    Entry, Function, Implementation, Input, ItemId, ItemKind, Model, Owner as Seated, Parameter, Placed, Reach, Region,
    Seat, TypeRef,
};
use lint_core::Site;
use lint_layout::{Name, Whole, Word, words};

use crate::transitions::context::Context;
use crate::transitions::endpoint::{Endpoint, Nature, Unnamed};
use crate::transitions::filler::Filler;
use crate::transitions::function::Function as Transitive;
use crate::transitions::ground::Ground;
use crate::transitions::resolved::Resolved;
use crate::transitions::wrapping::Wrapping;
use endpoints::Endpoints;
use peeling::Peeling;

#[derive(Debug)]
pub(crate) struct Reading<'file> {
    model: &'file Model,
    crates: BTreeMap<ItemId, &'file Name>,
    seated: BTreeMap<&'file Path, Vec<Placed<'file, &'file Function>>>,
    provided: BTreeMap<ItemId, Vec<Placed<'file, &'file Function>>>,
}

#[derive(Clone, Copy, Debug)]
struct Homed<'file> {
    name: &'file Name,
    krate: Option<&'file Name>,
}

#[derive(Clone, Debug)]
pub(crate) struct Scope<'look, 'file> {
    reading: &'look Reading<'file>,
    selfish: Nature,
    parameters: Vec<&'file Parameter>,
    output: Option<&'file TypeRef>,
}

impl<'file> From<&'file Model> for Reading<'file> {
    fn from(model: &'file Model) -> Reading<'file> {
        let defined: Vec<Placed<'file, &'file Function>> = model
            .functions()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter(|placed| placed.entity().definition().is_some())
            .collect();
        Reading {
            model,
            crates: model.items().map(|placed| (placed.entity().declared(), placed.krate())).collect(),
            seated: defined.iter().fold(BTreeMap::new(), |mut seated: BTreeMap<_, Vec<_>>, placed| {
                seated.entry(placed.site().path()).or_default().push(*placed);
                seated
            }),
            provided: defined
                .iter()
                .filter(|placed| placed.entity().owner() == Seated::TraitDecl)
                .fold(BTreeMap::new(), |mut provided: BTreeMap<_, Vec<_>>, placed| {
                    provided.entry(AsRef::<Seat>::as_ref(placed.entity()).owner()).or_default().push(*placed);
                    provided
                })
                .into_iter()
                .filter_map(|(owner, functions)| Some((owner?, functions)))
                .collect(),
        }
    }
}

impl<'file> Reading<'file> {
    const IMPLEMENTING: [Seated; 2] = [Seated::Inherent, Seated::TraitImpl];
    const OUTPUT: [Word<'static>; 1] = words!["Output"];

    pub(crate) fn contexts(&self) -> Vec<Context<'file>> {
        let implemented = self
            .model
            .implementations()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter(|placed| placed.entity().written().is_some())
            .map(|placed| self.implemented(placed));
        let traits = self
            .model
            .items()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter_map(|placed| {
                let item = self.model.declared(placed.entity().declared())?;
                (item.kind() == ItemKind::Trait).then(|| self.provided(placed, item.as_ref()))
            });
        let free = self
            .seated
            .values()
            .flatten()
            .filter(|placed| placed.entity().owner() == Seated::Free)
            .filter(|placed| placed.entity().entry() != Entry::Test)
            .map(|placed| {
                let scope = Scope { reading: self, selfish: Nature::Boxed, parameters: Vec::new(), output: None };
                Context::new(
                    placed.krate(),
                    Endpoint::from(Unnamed::Opaque),
                    Ground::from(*AsRef::<Region>::as_ref(placed.entity())),
                    Filler::Free,
                )
                .with_functions(scope.function(*placed))
            });
        let mut contexts: Vec<Context<'file>> = implemented.chain(traits).chain(free).collect();
        contexts
            .sort_by_key(|context| context.functions().first().map(|function| *AsRef::<Site<'file>>::as_ref(function)));
        contexts
    }

    fn implemented(&self, placed: Placed<'file, &'file Implementation>) -> Context<'file> {
        let implementation = placed.entity();
        let outer = Scope {
            reading: self,
            selfish: Nature::Boxed,
            parameters: AsRef::<[Parameter]>::as_ref(implementation).iter().collect(),
            output: None,
        };
        let selfish = implementation.subject_type().outlined(Endpoints::new(&outer)).or_opaque();
        let output = implementation
            .assigned()
            .iter()
            .find(|assigned| AsRef::<Name>::as_ref(*assigned).matches::<Whole>(&Self::OUTPUT).is_some())
            .map(AsRef::<TypeRef>::as_ref);
        let scope = Scope { selfish: selfish.nature(), output, ..outer };
        let filler = implementation.interface_name().cloned().map_or(Filler::Inherent, Filler::Implemented);
        let seated = self
            .seated
            .get(placed.site().path())
            .into_iter()
            .flatten()
            .filter(|function| function.entity().entry() != Entry::Test)
            .filter(|function| Self::IMPLEMENTING.contains(&function.entity().owner()))
            .filter(|function| {
                AsRef::<Seat>::as_ref(function.entity()).subject() == Some(implementation.subject_type())
            })
            .filter(|function| {
                AsRef::<Seat>::as_ref(function.entity()).implemented() == implementation.interface_name()
            })
            .filter_map(|function| scope.function(*function));
        let functions: Vec<Transitive<'file>> = seated.collect();
        Context::new(placed.krate(), selfish, Ground::from(*AsRef::<Region>::as_ref(implementation)), filler)
            .with_functions(functions)
    }

    fn provided(&self, placed: Placed<'file, &'file graph::ItemDeclaration>, name: &Name) -> Context<'file> {
        let scope = Scope { reading: self, selfish: Nature::Interface, parameters: Vec::new(), output: None };
        let functions: Vec<Transitive<'file>> = self
            .provided
            .get(&placed.entity().declared())
            .into_iter()
            .flatten()
            .filter(|function| function.entity().entry() != Entry::Test)
            .filter_map(|function| scope.function(*function))
            .collect();
        let region: &Region = placed.entity().as_ref();
        Context::new(
            placed.krate(),
            Endpoint::new(Nature::Interface, name.clone()),
            Ground::from(*region),
            Filler::Provided,
        )
        .with_functions(functions)
    }

    fn declared(&self, id: ItemId) -> Option<Homed<'file>> {
        Some(Homed { name: self.model.declared(id)?.as_ref(), krate: self.crates.get(&id).copied() })
    }
}

impl<'file> Scope<'_, 'file> {
    fn declared(&self, name: &Name) -> Option<&'file Parameter> {
        self.parameters.iter().rev().find(|parameter| AsRef::<Name>::as_ref(**parameter) == name).copied()
    }

    fn function(&self, placed: Placed<'file, &'file Function>) -> Option<Transitive<'file>> {
        let function = placed.entity();
        let definition = function.definition()?;
        let signature = function.signature();
        let inner = Scope {
            parameters: self.parameters.iter().copied().chain(AsRef::<[Parameter]>::as_ref(signature)).collect(),
            ..self.clone()
        };
        let takes = signature.inputs().iter().map(|input| inner.taken(input)).collect();
        let gives = signature
            .output()
            .written()
            .map_or(Resolved::Found(Endpoint::from(Unnamed::Unit)), |ty| ty.outlined(Endpoints::new(&inner)));
        let peeled = |ty: &TypeRef| ty.outlined(Peeling::new(&inner));
        Some(
            Transitive::new(placed.site(), AsRef::<Name>::as_ref(function).clone(), takes, gives)
                .shaped(signature.mutation(), definition.shape(), definition.branching())
                .wrapped(
                    signature.output().written().map_or(Wrapping::Plain, peeled),
                    signature.inputs().iter().map(|input| input.typed().map_or(Wrapping::Plain, peeled)).collect(),
                )
                .called(definition.invocations()),
        )
    }

    fn taken(&self, input: &Input) -> Resolved {
        input.typed().map_or(Resolved::Itself, |ty| ty.outlined(Endpoints::new(self)))
    }
}
