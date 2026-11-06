use std::io::{Stdout, Write};
use std::process::ExitCode;

use console_ports::Process;
use newtype::Counter;

use crate::{Failure, Harness, Passed, Workload};

#[derive(Debug)]
pub struct Gate<'out, O: Write> {
    out: &'out mut O,
    passed: Result<Passed, Failure>,
}

impl Gate<'_, Stdout> {
    pub fn answer(passing: impl for<'any> FnOnce(Gate<'any, Stdout>) -> Gate<'any, Stdout>) -> ExitCode {
        Process::answer(|_arguments, out, err| passing(Gate { out, passed: Ok(Passed::zero()) }).exit(err))
    }
}

impl<'out, O: Write> Gate<'out, O> {
    #[must_use]
    pub fn passing<W: Workload>(self) -> Gate<'out, O> {
        let Gate { out, passed } = self;
        let passed = passed.and_then(|count| {
            Harness::<W>::verified().and_then(|verified| Ok(write!(out, "{verified}")?)).map(|()| count.incremented())
        });
        Gate { out, passed }
    }

    fn exit(self, err: &mut impl Write) -> ExitCode {
        let Gate { out, passed } = self;
        passed.and_then(|count| Ok(writeln!(out, "{count}")?)).map_or_else(
            |failure| {
                writeln!(err, "{failure:?}").ok();
                ExitCode::FAILURE
            },
            |()| ExitCode::SUCCESS,
        )
    }
}
