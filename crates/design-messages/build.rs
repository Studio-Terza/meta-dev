use std::io::Write;

use console_ports::Process;
use exit::Exit;

#[derive(Debug)]
struct Watched;

fn main() -> Exit {
    Process::answer(|_arguments, out, err| {
        let watched = out.write_all(b"cargo::rerun-if-changed=migrations\n").map(|()| Watched);
        Exit::reported(watched, err)
    })
}
