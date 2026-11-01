use std::io;
use std::process::ExitCode;

use mutation_shard::Cli;

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1);
    Cli::answer(arguments, &mut io::stderr())
}
