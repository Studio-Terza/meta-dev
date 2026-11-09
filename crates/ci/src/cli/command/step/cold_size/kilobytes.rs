use std::fmt;

use file_ports::Size;
use newtype::{Arbitrary, Newtype, Wrapping};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kilobytes(u64);

impl Kilobytes {
    const KIBIBYTE: u64 = 1024;
}

impl From<Size> for Kilobytes {
    fn from(size: Size) -> Kilobytes {
        Kilobytes(size.into_inner().div_ceil(Kilobytes::KIBIBYTE))
    }
}

impl fmt::Display for Kilobytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{{\"kilobytes\": {}}}", self.0)
    }
}
