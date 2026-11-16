use std::collections::BTreeMap;
use std::fmt;

use lint_layout::Name;

use graph::{
    Evaluation, Exposure, Function, Held, Input, ItemId, ItemPath, Model, Owner, Parameter, Placed, Reach, Receiver,
    Region, Resolved, Returned, Seat, Site, Sited, Spelling, TypeRef,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Erased {
    Subject,
    Primitive(Name),
    Named(Resolved),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Spot {
    Receiver(Receiver),
    Inner,
    Typed(TypeRef<Erased>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key<'model> {
    name: &'model Name,
    inputs: Vec<Spot>,
    output: Option<TypeRef<Erased>>,
    bounds: Vec<TypeRef<Erased>>,
}

#[derive(Debug)]
pub struct Shaped<'model> {
    function: &'model Function,
    key: Key<'model>,
    owner: ItemId,
    site: Site<'model>,
}

#[derive(Clone, Copy, Debug)]
struct Subject<'model> {
    resolved: &'model Resolved,
}

#[derive(Debug, Default)]
pub struct Inners<'model>(BTreeMap<ItemId, Vec<&'model TypeRef>>);

impl<'model> Inners<'model> {
    pub fn read(model: &'model Model) -> Inners<'model> {
        let fields = model
            .holdings()
            .map(Placed::entity)
            .filter(|holding| matches!(holding.held(), Held::NewtypeField | Held::StructField))
            .filter_map(|holding| holding.declares().map(|owner| (owner, AsRef::<TypeRef>::as_ref(holding))));
        Inners(fields.fold(BTreeMap::new(), |mut inners: BTreeMap<ItemId, Vec<&'model TypeRef>>, (owner, ty)| {
            inners.entry(owner).or_default().push(ty);
            inners
        }))
    }

    fn only(&self, owner: ItemId) -> Option<&'model TypeRef> {
        self.0.get(&owner).filter(|fields| fields.len() == 1).and_then(|fields| fields.first().copied())
    }
}

impl<'model> Shaped<'model> {
    pub fn placed(placed: Placed<'model, &'model Function>, inners: &Inners<'model>) -> Option<Shaped<'model>> {
        let function = placed.entity();
        let seat: &Seat = function.as_ref();
        let region: &Region = function.as_ref();
        let owner = Some(function)
            .filter(|candidate| candidate.owner() == Owner::Inherent)
            .filter(|candidate| *AsRef::<Exposure>::as_ref(*candidate) != Exposure::Hidden)
            .filter(|candidate| *AsRef::<Evaluation>::as_ref(*candidate) == Evaluation::Runtime)
            .filter(|_| region.reach() == Reach::Shipped)
            .and(seat.owner())?;
        let subject = Subject { resolved: seat.subject()?.named()?.as_ref() };
        let inner = inners.only(owner).map(|ty| ty.mapped(&mut |sited| subject.erased(sited)));
        let signature = function.signature();
        let inputs = signature.inputs().iter().map(|input| subject.spot(input, inner.as_ref())).collect();
        let output = signature.output().written().map(|ty| ty.mapped(&mut |sited| subject.erased(sited)));
        let parameters = AsRef::<[Parameter]>::as_ref(signature).iter().flat_map(Parameter::bounds);
        let bounds = parameters.map(|ty| ty.mapped(&mut |sited| subject.erased(sited))).collect();
        Some(Shaped {
            function,
            key: Key { name: function.as_ref(), inputs, output, bounds },
            owner,
            site: placed.site(),
        })
    }

    pub fn owner(&self) -> ItemId {
        self.owner
    }

    pub fn grouped(methods: impl Iterator<Item = Shaped<'model>>) -> Vec<Vec<Shaped<'model>>> {
        let groups = methods.fold(BTreeMap::new(), |mut groups: BTreeMap<Key<'model>, Vec<Shaped<'model>>>, method| {
            groups.entry(method.key.clone()).or_default().push(method);
            groups
        });
        groups.into_values().collect()
    }

    fn returns_itself(&self) -> Option<()> {
        let named = self.key.output.as_ref()?.named()?;
        (*AsRef::<Erased>::as_ref(named) == Erased::Subject).then_some(())
    }
}

impl Subject<'_> {
    fn erased(self, sited: &Sited) -> Erased {
        let resolved: &Resolved = sited.as_ref();
        let named_self = resolved.parameter().and_then(Returned::named).filter(|found| *found == Returned::Itself);
        Some(resolved)
            .filter(|candidate| *candidate != self.resolved)
            .filter(|_| named_self.is_none())
            .map_or(Erased::Subject, |other| Self::named(sited, other))
    }

    fn named(sited: &Sited, resolved: &Resolved) -> Erased {
        let spelled = AsRef::<Spelling>::as_ref(sited).written().and_then(ItemPath::last);
        resolved
            .primitive()
            .and(spelled)
            .map_or_else(|| Erased::Named(resolved.clone()), |name| Erased::Primitive(name.clone()))
    }

    fn spot(self, input: &Input, inner: Option<&TypeRef<Erased>>) -> Spot {
        input.receiver().map_or_else(|| self.typed(input.typed(), inner), Spot::Receiver)
    }

    fn typed(self, ty: Option<&TypeRef>, inner: Option<&TypeRef<Erased>>) -> Spot {
        let erased = ty.map(|written| written.mapped(&mut |sited| self.erased(sited)));
        erased.filter(|found| Some(found) != inner).map_or(Spot::Inner, Spot::Typed)
    }
}

impl Spot {
    fn spelled(&self, written: &Input) -> String {
        match self {
            Spot::Receiver(receiver) => receiver.to_string(),
            Spot::Inner => String::from("Inner"),
            Spot::Typed(_) => written.typed().map(TypeRef::brief).map(|name| name.to_string()).unwrap_or_default(),
        }
    }
}

impl<'model> AsRef<Site<'model>> for Shaped<'model> {
    fn as_ref(&self) -> &Site<'model> {
        &self.site
    }
}

impl fmt::Display for Shaped<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signature = self.function.signature();
        let inputs: Vec<String> =
            self.key.inputs.iter().zip(signature.inputs()).map(|(spot, written)| spot.spelled(written)).collect();
        let spelled = signature.output().written().map(TypeRef::brief);
        let output = self.returns_itself().map_or(spelled, |()| Some(Name::from("Self")));
        let returned = output.map(|name| format!(" -> {name}")).unwrap_or_default();
        write!(formatter, "fn {}({}){returned}", self.key.name, inputs.join(", "))
    }
}
