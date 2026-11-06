use std::fmt;
use std::io::{self, Write};

use newtype::{Arbitrary, Counter, Newtype};

use super::failure::Failure;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Printed(usize);

#[derive(Debug)]
struct Counting<'out, O: Write> {
    out: &'out mut O,
    printed: Printed,
}

impl Printed {
    pub(crate) fn shown(out: &mut impl Write, report: &impl fmt::Display) -> Result<Printed, Failure> {
        let mut counting = Counting { out, printed: Printed::zero() };
        write!(counting, "{report}")?;
        Ok(counting.printed)
    }
}

impl<O: Write> Write for Counting<'_, O> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.out.write(bytes)?;
        self.printed = self.printed.plus(Printed::new(written));
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

impl fmt::Display for Printed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "printed {} bytes", self.count())
    }
}
