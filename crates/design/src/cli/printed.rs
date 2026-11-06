use std::fmt;
use std::io::Write;

use newtype::{Arbitrary, Newtype};

use super::failure::Failure;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Printed(usize);

impl Printed {
    pub(crate) fn shown(out: &mut impl Write, report: &impl fmt::Display) -> Result<Printed, Failure> {
        let mut rendered = Vec::new();
        write!(rendered, "{report}")?;
        out.write_all(&rendered)?;
        Ok(Printed::new(rendered.len()))
    }
}
