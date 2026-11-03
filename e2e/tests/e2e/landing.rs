use std::error::Error;
use std::path::PathBuf;

use crate::fake::Fake;
use crate::ports::desk::{Desk, Scope};
use crate::ports::report::{Fraction, Report};
use crate::ports::world::World;

#[derive(Debug, Default)]
struct Spend {
    runs: usize,
    crate_builds: usize,
    mutant_crates: usize,
}

impl Spend {
    fn counted(&mut self, scope: &Scope, world: &World) {
        self.runs += 1;
        self.crate_builds += scope.built(world.names().cloned()).len();
        self.mutant_crates += scope.mutated().len();
    }

    fn reported(&self, report: &Report, path: &str, changes: usize) -> std::io::Result<()> {
        report.line(&format!("{path}_runs"), Fraction::of(self.runs, changes))?;
        report.line(&format!("{path}_crate_builds"), Fraction::of(self.crate_builds, changes))?;
        report.line(&format!("{path}_mutant_crates"), Fraction::of(self.mutant_crates, changes))
    }
}

#[derive(Debug)]
struct Runs<'world> {
    world: &'world World,
    metadata: String,
    desk: Desk,
}

impl Runs<'_> {
    const PULL_REQUEST: &'static str = "pull_request";
    const PUSH: &'static str = "push";
    const MAIN: &'static str = "main";
    const PROJECT: &'static str = "project/e2e";
    const NO_BASE: &'static str = "";

    fn spent(&self, spend: &mut Spend, changed: &[PathBuf], event: &str, base: &str) -> Result<(), Box<dyn Error>> {
        let scope = self.desk.scope(&Fake::modifying(&self.metadata, changed), event, base)?;
        spend.counted(&scope, self.world);
        Ok(())
    }
}

#[test]
fn a_project_landed_through_its_branch_against_its_changes_sent_to_main_one_by_one() -> Result<(), Box<dyn Error>> {
    const PROJECT_SIZE: usize = 4;
    let world = World::of_this_repository()?;
    let runs = Runs { world: &world, metadata: world.metadata(), desk: Desk::default() };
    let names: Vec<&String> = world.names().collect();
    let (mut project, mut direct) = (Spend::default(), Spend::default());
    for members in names.chunks(PROJECT_SIZE) {
        let changed: Vec<PathBuf> = members.iter().map(|name| world.source_of(name)).collect();
        for one in changed.chunks(1) {
            runs.spent(&mut project, one, Runs::PULL_REQUEST, Runs::PROJECT)?;
            runs.spent(&mut direct, one, Runs::PULL_REQUEST, Runs::MAIN)?;
            runs.spent(&mut direct, one, Runs::PUSH, Runs::NO_BASE)?;
        }
        runs.spent(&mut project, &changed, Runs::PULL_REQUEST, Runs::MAIN)?;
        runs.spent(&mut project, &changed, Runs::PUSH, Runs::NO_BASE)?;
    }
    let report = Report::named("landing");
    report.line("changes", names.len())?;
    report.line("crates", world.total())?;
    project.reported(&report, "project", names.len())?;
    direct.reported(&report, "direct", names.len())?;
    Ok(())
}
