use std::ffi::OsString;
use std::fmt;
use std::num::NonZeroU16;
use std::str::FromStr;

use crate::cli::command::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shard {
    index: u16,
    count: NonZeroU16,
}

impl Shard {
    const SEPARATOR: char = '/';
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
        text.char_indices()
            .find(|(_, letter)| *letter == Self::SEPARATOR)
            .and_then(|(at, _)| text.split_at_checked(at))
            .and_then(|(index, rest)| {
                Some(Shard { index: index.parse().ok()?, count: rest.get(Self::SEPARATOR.len_utf8()..)?.parse().ok()? })
            })
            .filter(|shard| shard.index < shard.count.get())
            .ok_or(Failure::Usage)
    }
}

impl fmt::Display for Shard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}{}", self.index, Self::SEPARATOR, self.count)
    }
}
