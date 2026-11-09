use std::error::Error;

use crate::fake::{Change, Edit, Fake};
use crate::ports::desk::Desk;
use crate::ports::report::{Fraction, Report};
use crate::ports::world::World;

#[derive(Debug, Default)]
struct Verdicts {
    pairs: usize,
    passed: usize,
    disagreements: usize,
}

impl Verdicts {
    fn counted(self, passed: bool, expected: bool) -> Verdicts {
        Verdicts {
            pairs: self.pairs + 1,
            passed: self.passed + usize::from(passed),
            disagreements: self.disagreements + usize::from(passed != expected),
        }
    }

    fn reported(&self, report: &Report, kind: &str) -> std::io::Result<()> {
        report.line(&format!("{kind}_pairs"), self.pairs)?;
        report.line(&format!("{kind}_owner_share"), Fraction::of(self.pairs - self.passed, self.pairs))?;
        report.line(&format!("{kind}_disagreements"), self.disagreements)
    }
}

#[derive(Debug, Default)]
struct Tallies {
    goldens: Verdicts,
    scenarios: Verdicts,
}

#[test]
fn goldens_and_new_scenarios_pass_only_inside_the_reach_of_the_rest_of_the_change() -> Result<(), Box<dyn Error>> {
    const BASE: &str = "project/e2e";
    let world = World::of_this_repository()?;
    let metadata = world.metadata();
    let desk = Desk::default();
    let tallies = world.names().fold(Tallies::default(), |tallies, changed| {
        let source = world.source_of(changed);
        let reached = world.reached(std::slice::from_ref(&source));
        world.names().fold(tallies, |so_far, owner| {
            let golden = world.below(owner, &["tests", "goldens", "e2e.txt"]);
            let scenario = world.below(owner, &["tests", "e2e", "added.rs"]);
            let expected = world.owners(&golden).is_subset(&reached);
            let with_golden = Fake::new(
                &metadata,
                vec![Change::new(Edit::Modified, source.clone()), Change::new(Edit::Modified, golden)],
            );
            let with_scenario = Fake::new(
                &metadata,
                vec![Change::new(Edit::Modified, source.clone()), Change::new(Edit::Added, scenario)],
            );
            Tallies {
                goldens: so_far.goldens.counted(desk.labels_pass(&with_golden, BASE), expected),
                scenarios: so_far.scenarios.counted(desk.labels_pass(&with_scenario, BASE), expected),
            }
        })
    });
    let report = Report::named("golden-reach");
    tallies.goldens.reported(&report, "golden")?;
    tallies.scenarios.reported(&report, "scenario")?;
    Ok(())
}
