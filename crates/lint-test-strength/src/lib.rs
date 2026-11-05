mod pairing;

use lint_core::{Headline, Report, Violation};
use lint_transition_graph::{ChangeLint, TypeSystem};

use pairing::Pairing;

#[derive(Debug)]
pub struct TestStrength;

impl ChangeLint for TestStrength {
    fn report_change<'file>(before: &TypeSystem<'_, 'file>, after: &TypeSystem<'_, 'file>) -> Report<'file> {
        Report::new(
            Headline::new(
                "a change never weakens a test: every claim a test made before the change (a subject in an accepted set of values, for every value of its domains) is entailed by a claim of its package after it, with the same subject, an accepted set inside the old one and domains that cover the old ones",
            ),
            Pairing::across(before.tests(), after.tests()).lost().into_iter().map(Violation::from).collect(),
        )
    }
}
