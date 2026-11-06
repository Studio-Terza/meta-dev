use std::io::Write;
use std::process::ExitCode;

use super::failure::Failure;

#[derive(Debug)]
pub struct Streams<'streams, O: Write, E: Write> {
    out: &'streams mut O,
    err: &'streams mut E,
}

impl<'streams, O: Write, E: Write> Streams<'streams, O, E> {
    pub fn new(out: &'streams mut O, err: &'streams mut E) -> Streams<'streams, O, E> {
        Streams { out, err }
    }

    pub fn out(&mut self) -> &mut O {
        self.out
    }

    pub fn refused(&mut self, failure: &Failure) -> ExitCode {
        writeln!(self.err, "{failure}").ok();
        ExitCode::FAILURE
    }
}
