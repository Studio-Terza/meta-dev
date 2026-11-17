use crate::cli::command::Telling;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use super::Step;
use super::boundary::Shard;
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::ports::Ports;

#[derive(Debug)]
pub struct MutantsShard {
    shard: Shard,
    output: PathBuf,
}

impl MutantsShard {
    const PASSED_ON: [&'static str; 2] = ["--", "--no-fail-fast"];
    const ARGUMENTS: usize = 2;
}

impl Step for MutantsShard {
    fn from_arguments(arguments: Vec<OsString>) -> Result<MutantsShard, Failure<'static>> {
        match <[OsString; Self::ARGUMENTS]>::try_from(arguments) {
            Ok([shard, output]) => Ok(MutantsShard {
                shard: shard.to_str().ok_or(Failure::Usage)?.parse()?,
                output: PathBuf::from(output),
            }),
            Err(_) => Err(Failure::Usage),
        }
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let shard = self.shard.arguments();
        let selection: Vec<&OsStr> = shard.iter().map(OsString::as_os_str).collect();
        let invocation = CargoMutants::invocation(&self.output, &selection, &Self::PASSED_ON.map(OsStr::new));
        Ok(Telling::ended(
            ports
                .run(&invocation)
                .map(|run| run.exit().excusing(&[CargoMutants::MISSED, CargoMutants::TIMEOUT]))
                .map_err(Failure::starting(&invocation)),
        ))
    }
}
