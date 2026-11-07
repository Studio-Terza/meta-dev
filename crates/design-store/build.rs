use std::io::{self, Write};

use console_ports::Process;
use exit::Exit;

#[derive(Debug)]
struct Watched;

fn main() -> Exit<Watched, io::Error> {
    Exit::from(Process::answer(|_arguments, out, _err| {
        out.write_all(b"cargo::rerun-if-changed=migrations\n").map(|()| Watched)
    }))
}
