use std::convert::Infallible;
use std::str::FromStr;

use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Characters(usize);

impl FromStr for Characters {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Characters, Infallible> {
        Ok(Characters(text.chars().count()))
    }
}
