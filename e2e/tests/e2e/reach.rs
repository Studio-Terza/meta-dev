use std::collections::BTreeSet;
use std::error::Error;

use crate::fake::Fake;
use crate::ports::desk::Desk;
use crate::ports::report::{Fraction, Report};
use crate::ports::world::World;

#[derive(Debug, Default)]
struct Tally {
    built: usize,
    misses: usize,
    extras: usize,
}

impl Tally {
    fn counted(self, observed: &BTreeSet<String>, expected: &BTreeSet<String>) -> Tally {
        Tally {
            built: self.built + observed.len(),
            misses: self.misses + expected.difference(observed).count(),
            extras: self.extras + observed.difference(expected).count(),
        }
    }
}

#[test]
fn a_pull_request_into_a_project_branch_builds_the_reach_of_its_one_changed_crate() -> Result<(), Box<dyn Error>> {
    let world = World::of_this_repository()?;
    let metadata = world.metadata();
    let desk = Desk::default();
    let tally = world.names().try_fold(Tally::default(), |tally, name| -> Result<Tally, Box<dyn Error>> {
        let changed = [world.source_of(name)];
        let scope = desk.scope(&Fake::modifying(&metadata, &changed), "pull_request", "project/e2e")?;
        Ok(tally.counted(&scope.built(world.names().cloned()), &world.reached(&changed)))
    })?;
    let report = Report::named("reach");
    report.line("crates", world.total())?;
    report.line("share", Fraction::of(tally.built, world.total() * world.total()))?;
    report.line("misses", tally.misses)?;
    report.line("extras", tally.extras)?;
    Ok(())
}
