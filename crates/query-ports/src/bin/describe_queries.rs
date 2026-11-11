use console_ports::Process;
use exit::Exit;
use query_ports::QueryCache;

fn main() -> Exit {
    Process::answer(|arguments, out, err| {
        let prepared = QueryCache::try_from(arguments).and_then(|cache| cache.prepared(out));
        Exit::reported(prepared, err)
    })
}
