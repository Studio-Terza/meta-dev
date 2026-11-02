use std::error::Error;

use crate::desk::Desk;
use crate::fake::{Change, Edit, Fake};
use crate::report::{Fraction, Report};
use crate::world::World;

#[derive(Debug, Default)]
struct Verdicts {
    pairs: usize,
    owner: usize,
    disagreements: usize,
}

impl Verdicts {
    fn counted(&mut self, passed: bool, expected: bool) {
        self.pairs += 1;
        self.owner += usize::from(!passed);
        self.disagreements += usize::from(passed != expected);
    }

    fn reported(&self, report: &Report, kind: &str) -> std::io::Result<()> {
        report.line(&format!("{kind}_pairs"), self.pairs)?;
        report.line(&format!("{kind}_owner_share"), Fraction::of(self.owner, self.pairs))?;
        report.line(&format!("{kind}_disagreements"), self.disagreements)
    }
}

#[test]
fn goldens_and_new_scenarios_pass_only_inside_the_reach_of_the_rest_of_the_change() -> Result<(), Box<dyn Error>> {
    const BASE: &str = "project/e2e";
    let world = World::of_this_repository()?;
    let metadata = world.metadata();
    let desk = Desk::new()?;
    let (mut goldens, mut scenarios) = (Verdicts::default(), Verdicts::default());
    for changed in world.names() {
        let source = world.source_of(changed);
        let reached = world.reached(std::slice::from_ref(&source));
        for owner in world.names() {
            let golden = world.below(owner, &["tests", "goldens", "e2e.txt"]);
            let scenario = world.below(owner, &["tests", "e2e", "added.rs"]);
            let expected = world.owners(&golden).is_subset(&reached);
            let with_golden = Fake::new(
                &metadata,
                vec![Change::new(Edit::Modified, source.clone()), Change::new(Edit::Modified, golden)],
            );
            goldens.counted(desk.labels_pass(&with_golden, BASE), expected);
            let with_scenario = Fake::new(
                &metadata,
                vec![Change::new(Edit::Modified, source.clone()), Change::new(Edit::Added, scenario)],
            );
            scenarios.counted(desk.labels_pass(&with_scenario, BASE), expected);
        }
    }
    let report = Report::named("golden-reach");
    goldens.reported(&report, "golden")?;
    scenarios.reported(&report, "scenario")?;
    Ok(())
}
