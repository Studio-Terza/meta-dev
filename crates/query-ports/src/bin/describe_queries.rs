use std::process::ExitCode;

use console_ports::{Answer, Process};
use query_ports::QueryCache;

fn main() -> ExitCode {
    Process::tell(|arguments| {
        let prepared = QueryCache::try_from(arguments).and_then(|cache| cache.prepared());
        let shown = prepared.as_ref().map_or_default(ToString::to_string);
        Answer::reported(prepared).showing(shown)
    })
}
