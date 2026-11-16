use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;

use grammar_shard::Sharding;

use crate::cli::command::failure::Failure;
use crate::cli::ports::Text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shard(Sharding);

impl Shard {
    const FLAG: &'static str = "--shard";
    const SHARDING: [&'static str; 2] = ["--sharding", "round-robin"];

    pub(crate) fn arguments(self) -> [OsString; 4] {
        let [sharding, order] = Self::SHARDING.map(OsString::from);
        [OsString::from(Self::FLAG), OsString::from(self.to_string()), sharding, order]
    }
}

impl FromStr for Shard {
    type Err = Failure<'static>;

    fn from_str(text: &str) -> Result<Shard, Failure<'static>> {
        text.parse().map(Shard).map_err(|_unsharded| Failure::Usage)
    }
}

impl fmt::Display for Shard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exact<'name>(&'name Text);

impl Exact<'_> {
    const SPECIAL: &'static str = "\\.+*?()|[]{}^$#&-~";
}

impl<'name> From<&'name Text> for Exact<'name> {
    fn from(name: &'name Text) -> Exact<'name> {
        Exact(name)
    }
}

impl fmt::Display for Exact<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let escaped: String = self
            .0
            .as_ref()
            .chars()
            .flat_map(|letter| Self::SPECIAL.contains(letter).then_some('\\').into_iter().chain([letter]))
            .collect();
        write!(formatter, "^{escaped}$")
    }
}

impl From<Exact<'_>> for OsString {
    fn from(exact: Exact<'_>) -> OsString {
        OsString::from(exact.to_string())
    }
}
