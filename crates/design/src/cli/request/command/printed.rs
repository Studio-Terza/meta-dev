use std::fmt;
use std::io::Write;

use newtype::{Arbitrary, Newtype};

use crate::cli::failure::Failure;
use crate::cli::output::Line;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Printed(usize);

impl Printed {
    pub(crate) fn lines(out: &mut impl Write, lines: &[Line]) -> Result<Printed, Failure> {
        let mut rendered = Vec::new();
        lines.iter().try_for_each(|line| writeln!(rendered, "{line}"))?;
        out.write_all(&rendered)?;
        Ok(Printed::new(rendered.len()))
    }

    pub(crate) fn rendered(out: &mut impl Write, rendered: &impl fmt::Display) -> Result<Printed, Failure> {
        let mut bytes = Vec::new();
        write!(bytes, "{rendered}")?;
        out.write_all(&bytes)?;
        Ok(Printed::new(bytes.len()))
    }
}
