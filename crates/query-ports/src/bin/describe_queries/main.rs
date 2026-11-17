use std::process::ExitCode;

use console_ports::Process;
use query_ports::QueryCache;

fn main() -> ExitCode {
    Process::tell(QueryCache::answered)
}
