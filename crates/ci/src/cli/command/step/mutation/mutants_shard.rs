use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;

use super::Step;
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct MutantsShard {
    shard: OsString,
    output: PathBuf,
}

impl MutantsShard {
    const SHARDING: [&'static str; 2] = ["--sharding", "round-robin"];
    const PASSED_ON: [&'static str; 2] = ["--", "--no-fail-fast"];
    const SHARD: &'static str = "--shard";
}

impl Step for MutantsShard {
    fn from_arguments(arguments: &[OsString]) -> Result<MutantsShard, Failure> {
        match arguments {
            [shard, output] => Ok(MutantsShard { shard: shard.clone(), output: PathBuf::from(output) }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure> {
        let selection: Vec<&OsStr> = [OsStr::new(Self::SHARD), self.shard.as_os_str()]
            .into_iter()
            .chain(Self::SHARDING.iter().map(OsStr::new))
            .collect();
        let invocation = CargoMutants::invocation(&self.output, &selection, &Self::PASSED_ON);
        ports
            .run(&invocation)
            .map(|run| run.exit().excusing(&[CargoMutants::MISSED, CargoMutants::TIMEOUT]))
            .map_err(Failure::starting(&invocation))
    }
}
