use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use graph::{Domain, Item, Mention, Model, PathUse, Placed, Resolved};
use lint_core::{Fix, Headline, Message, Problem, Report, Violation};
use lint_file::ModelLint;
use lint_layout::Site;

use crate::LawRunners;
use crate::label::Label;

#[derive(Debug)]
pub struct HandWrittenLaws;

#[derive(Debug)]
struct Emitted<'model> {
    macros: Vec<&'model Resolved>,
    files: BTreeSet<PathBuf>,
}

impl HandWrittenLaws {
    const REFUSED: Message = Message::new(
        Problem::new("a law the law runner generates, written by hand"),
        Fix::new("label the type instead (derive Arbitrary, implement Monoid) and run lint laws to regenerate"),
    );
    const UNDECLARED: Message = Message::new(
        Problem::new(
            "this crate's generated law runner tests/generated/laws.rs is in no test binary, so its laws never run",
        ),
        Fix::new("add #[path = \"generated/laws.rs\"] mod generated_laws; to one of the crate's integration tests"),
    );
}

impl ModelLint for HandWrittenLaws {
    fn report(model: &Model) -> Report<'_> {
        let emitted = Emitted::from(model);
        let runners = LawRunners::from(model);
        let invoking: Vec<Placed<'_, &PathUse>> = model.uses().filter_map(|placed| emitted.invoked(placed)).collect();
        let read: BTreeSet<&Path> = invoking.iter().map(|placed| placed.site().path()).collect();
        let refused = invoking.iter().filter_map(|placed| emitted.refused(model, *placed));
        let undeclared = runners
            .undeclared(&read)
            .into_iter()
            .map(|root| Violation::new(Site::of_file(root), HandWrittenLaws::UNDECLARED));
        Report::new(
            Headline::new(
                "laws are labels: the law macros lint laws emits appear only in the runners it generates, and each runner is declared in one of its crate's test binaries, so the graph owns every such law and every one runs",
            ),
            refused.chain(undeclared).collect(),
        )
    }
}

impl<'model> Emitted<'model> {
    fn invoked(&self, placed: Placed<'model, &'model PathUse>) -> Option<Placed<'model, &'model PathUse>> {
        Some(placed)
            .filter(|found| found.entity().mention() == Mention::Invoked)
            .filter(|found| self.macros.contains(&found.entity().resolved()))
    }

    fn refused(&self, model: &Model, placed: Placed<'model, &'model PathUse>) -> Option<Violation<'model>> {
        let used = placed.entity();
        let declaring = used.resolved().crate_item().and_then(|item| model.declared(item)).and_then(Item::krate);
        declaring.filter(|krate| *krate == placed.krate()).map_or(Some(()), |_| None)?;
        self.files.get(placed.site().path()).map_or(Some(()), |_| None)?;
        Some(Violation::new(placed.site(), HandWrittenLaws::REFUSED))
    }
}

impl<'model> From<&'model Model> for Emitted<'model> {
    fn from(model: &'model Model) -> Emitted<'model> {
        Emitted {
            macros: [Label::GENERATES, Label::MONOID]
                .into_iter()
                .filter_map(|spelled| model.exported(Domain::Macros, &spelled.into_iter().collect()))
                .collect(),
            files: LawRunners::from(model).paths(),
        }
    }
}
