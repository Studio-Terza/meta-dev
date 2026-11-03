use std::error::Error;

use crate::fake::Fake;
use crate::ports::desk::Desk;
use crate::ports::report::{Fraction, Report};
use crate::ports::world::World;

#[test]
fn a_pull_request_into_a_project_branch_builds_the_reach_of_its_one_changed_crate() -> Result<(), Box<dyn Error>> {
    let world = World::of_this_repository()?;
    let metadata = world.metadata();
    let desk = Desk::default();
    let (mut built, mut misses, mut extras) = (0, 0, 0);
    for name in world.names() {
        let changed = [world.source_of(name)];
        let scope = desk.scope(&Fake::modifying(&metadata, &changed), "pull_request", "project/e2e")?;
        let observed = scope.built(world.names().cloned());
        let expected = world.reached(&changed);
        built += observed.len();
        misses += expected.difference(&observed).count();
        extras += observed.difference(&expected).count();
    }
    let report = Report::named("reach");
    report.line("crates", world.total())?;
    report.line("share", Fraction::of(built, world.total() * world.total()))?;
    report.line("misses", misses)?;
    report.line("extras", extras)?;
    Ok(())
}
