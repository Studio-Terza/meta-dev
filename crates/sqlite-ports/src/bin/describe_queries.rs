use console_ports::Process;
use exit::Exit;
use sqlite_ports::{QueryCache, Uncached};

fn main() -> Exit<(), Uncached> {
    Exit::from(Process::answer(|arguments, out, _err| QueryCache::try_from(arguments)?.prepared(out)))
}
