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
                "a change never weakens a test: every check a test made before the change (an assertion's relation, the values it pins, the cases it ranges over, or a generated law) is matched, one to one, by a check at least as strong in the same test after it, or in a test the change added to the same package when the test was renamed, moved or split; strength is the product order of relation (equality above a pattern above a bare condition, an inequality above a bare condition, a law only by its own kind), pinned values and domain (generated cases above cases shipped code enumerates above listed rows, rows by cases and values)",
            ),
            Pairing::across(before.tests(), after.tests()).lost().into_iter().map(Violation::from).collect(),
        )
    }
}
