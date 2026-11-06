mod fixture;

use std::ffi::OsString;

use design::{Cli, Failure, Printed};
use port_core::{Calls, Run};
use transcript_ports::transcripts_ops::{ReadFrom, Sessions};
use workload::{Complexity, Scale, Workload};

use fixture::Fixture;

struct Ingest;

struct Show;

struct Reading;

impl Reading {
    fn run(run: &Run, fixture: Fixture, arguments: Vec<OsString>) -> Result<Printed, Failure> {
        Cli::new(&fixture.environment, fixture.memory).report(run, arguments.into_iter(), &mut Vec::new())
    }
}

impl Workload for Ingest {
    type Fixture = Fixture;
    type Done = Printed;
    type Failure = Failure;
    type Scaled = ReadFrom;
    type Listed = Sessions;

    const CLASS: Complexity = Complexity::Linear;

    fn fixture(scale: Scale) -> Fixture {
        Fixture::at(scale)
    }

    fn inputs(scale: Scale) -> Calls {
        Calls::new(Fixture::sessions_at(scale))
    }

    fn run(run: &Run, fixture: Fixture) -> Result<Printed, Failure> {
        Reading::run(run, fixture, vec![OsString::from("ingest")])
    }
}

impl Workload for Show {
    type Fixture = Fixture;
    type Done = Printed;
    type Failure = Failure;
    type Scaled = ReadFrom;
    type Listed = Sessions;

    const CLASS: Complexity = Complexity::Linear;

    fn fixture(scale: Scale) -> Fixture {
        Fixture::at(scale)
    }

    fn inputs(scale: Scale) -> Calls {
        Calls::new(Fixture::sessions_at(scale))
    }

    fn run(run: &Run, fixture: Fixture) -> Result<Printed, Failure> {
        let arguments = vec![OsString::from("show"), OsString::from(fixture.last_message())];
        Reading::run(run, fixture, arguments)
    }
}

workload::workload!(Ingest, Show);
