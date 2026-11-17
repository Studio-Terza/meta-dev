use std::collections::BTreeSet;
use std::path::PathBuf;

use graph::{Domain, Item, Mention, Model, PathUse, Placed, Resolved};
use lint_core::{Fix, Headline, Message, Problem, Report, Violation};
use lint_file::ModelLint;

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
}

impl ModelLint for HandWrittenLaws {
    fn report(model: &Model) -> Report<'_> {
        let emitted = Emitted::from(model);
        Report::new(
            Headline::new(
                "laws are labels: the law macros lint laws emits appear only in the files it generates, so the graph owns every such law",
            ),
            model.uses().filter_map(|placed| emitted.refused(model, placed)).collect(),
        )
    }
}

impl<'model> Emitted<'model> {
    fn refused(&self, model: &Model, placed: Placed<'model, &'model PathUse>) -> Option<Violation<'model>> {
        let used = Some(placed.entity()).filter(|used| used.mention() == Mention::Invoked)?;
        self.macros.iter().find(|wanted| **wanted == used.resolved())?;
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
