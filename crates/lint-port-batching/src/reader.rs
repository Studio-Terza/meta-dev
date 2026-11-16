mod finding;

use std::collections::BTreeMap;
use std::path::Path;

use graph::{Function, Implementation, Model, Parameter, Placed, Reached, Repetition, Seat};
use lint_core::{Site, Violation};
use lint_layout::{LineNumber, Name};

use crate::ports::{Ported, Ports, Typing};
use finding::{Finding, Port};

#[derive(Debug)]
pub(crate) struct Reader<'run, 'model> {
    ports: &'run Ports<'model>,
    impls: BTreeMap<&'model Path, Vec<&'model Implementation>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Spot<'model> {
    site: Site<'model>,
    port: Port,
}

#[derive(Debug, Default)]
pub(crate) struct Found<'model>(BTreeMap<Spot<'model>, Finding>);

#[derive(Debug)]
pub(crate) struct Flagged<'model> {
    spot: Spot<'model>,
    finding: Finding,
}

#[derive(Clone, Copy, Debug)]
struct Body<'run, 'model> {
    function: &'model Function,
    path: &'model Path,
    typing: Typing<'run, 'model>,
    held: Option<&'run std::collections::BTreeSet<&'model Name>>,
}

impl<'run, 'model> Reader<'run, 'model> {
    pub(crate) fn new(model: &'model Model, ports: &'run Ports<'model>) -> Reader<'run, 'model> {
        let impls = model.implementations().fold(BTreeMap::new(), |mut impls: BTreeMap<_, Vec<_>>, placed| {
            impls.entry(<&Path>::from(placed.site())).or_default().push(placed.entity());
            impls
        });
        Reader { ports, impls }
    }

    pub(crate) fn found(&self, placed: Placed<'model, &'model Function>) -> Vec<Flagged<'model>> {
        let function = placed.entity();
        let seat: &Seat = function.as_ref();
        let implemented = seat.subject().and_then(|subject| {
            self.impls.get(<&Path>::from(placed.site()))?.iter().find(|found| found.subject_type() == subject).copied()
        });
        let generics: Vec<&Parameter> = implemented
            .map_or(&[][..], |found| AsRef::<[Parameter]>::as_ref(found))
            .iter()
            .chain(AsRef::<[Parameter]>::as_ref(function.signature()))
            .collect();
        let body = Body {
            function,
            path: <&Path>::from(placed.site()),
            typing: self.ports.typing(&generics),
            held: seat.owner().and_then(|owner| self.ports.fields(owner)),
        };
        let repetitions = function.definition().map_or(&[][..], |definition| definition.repetitions());
        repetitions
            .iter()
            .filter(|repetition| repetition.method().and_then(|method| self.ports.paces(method)).is_none())
            .filter_map(|repetition| body.spotted(repetition))
            .collect()
    }
}

impl<'model> Body<'_, 'model> {
    fn spotted(self, repetition: &Repetition) -> Option<Flagged<'model>> {
        let reached = repetition.reach();
        let port = reached.index().map_or_else(
            || self.field(reached),
            |index| {
                let input = index.entry(self.function.signature().inputs())?.typed()?;
                Some(input.outlined(self.typing))
                    .filter(|ported| *ported == Ported::Port)
                    .map(|_| Port::Binding(AsRef::<Name>::as_ref(reached).clone()))
            },
        )?;
        let site = Site::new(self.path, *AsRef::<LineNumber>::as_ref(repetition));
        Some(Flagged {
            spot: Spot { site, port: port.clone() },
            finding: Finding::new(port, repetition.repeating().clone()),
        })
    }

    fn field(self, reached: &Reached) -> Option<Port> {
        let name: &Name = reached.as_ref();
        self.held.filter(|fields| fields.contains(name)).map(|_| Port::Field(name.clone()))
    }
}

impl<'model> FromIterator<Flagged<'model>> for Found<'model> {
    fn from_iter<Flags: IntoIterator<Item = Flagged<'model>>>(flags: Flags) -> Found<'model> {
        Found(flags.into_iter().fold(BTreeMap::new(), |mut found, flagged| {
            found.entry(flagged.spot).or_insert(flagged.finding);
            found
        }))
    }
}

impl<'model> IntoIterator for Found<'model> {
    type Item = Violation<'model>;
    type IntoIter = std::vec::IntoIter<Violation<'model>>;

    fn into_iter(self) -> std::vec::IntoIter<Violation<'model>> {
        let violations: Vec<Violation<'model>> =
            self.0.into_iter().map(|(spot, finding)| Violation::new(spot.site, finding)).collect();
        violations.into_iter()
    }
}
