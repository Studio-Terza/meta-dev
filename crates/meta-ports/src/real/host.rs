pub use std::process::ExitCode as Exit;

pub fn arguments() -> Vec<String> {
    std::env::args().skip(1).collect()
}

pub fn exit(code: u8) -> Exit {
    Exit::from(code)
}
