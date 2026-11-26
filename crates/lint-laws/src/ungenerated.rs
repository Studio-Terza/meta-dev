use graph::Model;
use lint_core::{Fix, Headline, Message, Problem, Report, Violation};

use crate::LawRunners;

#[derive(Debug)]
pub struct UngeneratedLaws;

impl UngeneratedLaws {
    const UNGENERATED: Message = Message::new(
        Problem::new("a Semigroup or Monoid the law runner cannot generate, so its laws never run"),
        Fix::new("derive newtype::Arbitrary on the type and run lint laws to regenerate"),
    );

    #[must_use]
    pub fn listed<'model>(model: &'model Model, runners: &LawRunners<'model>) -> Report<'model> {
        Report::new(
            Headline::new(
                "report only, ungenerated laws: every public Semigroup and Monoid is labelled, so the law runner generates its laws from its Arbitrary; a lawful type with no generator is listed here until it is blocking at zero",
            ),
            model
                .items()
                .filter(|placed| runners.ungenerated().contains(&placed.entity().declared()))
                .map(|placed| Violation::new(placed.site(), UngeneratedLaws::UNGENERATED))
                .collect(),
        )
    }
}
