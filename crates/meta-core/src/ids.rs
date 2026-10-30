use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformed(String);

impl Malformed {
    pub(crate) fn because(why: impl Into<String>) -> Self {
        Self(why.into())
    }
}

impl fmt::Display for Malformed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

macro_rules! counted_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const PREFIX: &str = $prefix;

            pub fn after(count: usize) -> Self {
                Self(count as u64 + 1)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}{}", Self::PREFIX, self.0)
            }
        }

        impl FromStr for $name {
            type Err = Malformed;

            fn from_str(text: &str) -> Result<Self, Malformed> {
                text.strip_prefix(Self::PREFIX)
                    .and_then(|digits| digits.parse().ok())
                    .map(Self)
                    .ok_or_else(|| Malformed(format!("{text:?} is not a {}<number> id", Self::PREFIX)))
            }
        }
    };
}

macro_rules! text {
    ($name:ident, $what:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl TryFrom<String> for $name {
            type Error = Malformed;

            fn try_from(text: String) -> Result<Self, Malformed> {
                let trimmed = text.trim();
                match trimmed.is_empty() {
                    true => Err(Malformed(format!("{} is empty", $what))),
                    false => Ok(Self(trimmed.to_owned())),
                }
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = Malformed;

            fn from_str(text: &str) -> Result<Self, Malformed> {
                Self::try_from(text.to_owned())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

counted_id!(TaskId, "task-");
counted_id!(LeaseId, "lease-");

text!(Title, "title");
text!(AgentName, "agent name");
text!(Reason, "reason");
text!(Sha, "sha");
text!(CheckName, "check name");
text!(Origin, "origin");
text!(Branch, "branch");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrNumber(u64);

impl fmt::Display for PrNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{}", self.0)
    }
}

impl PrNumber {
    pub fn digits(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for PrNumber {
    type Err = Malformed;

    fn from_str(text: &str) -> Result<Self, Malformed> {
        text.trim_start_matches('#').parse().map(Self).map_err(|_| Malformed(format!("{text:?} is not a PR number")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn from_unix_seconds(seconds: u64) -> Self {
        Self(seconds)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunCount(u64);

impl RunCount {
    pub fn of(count: usize) -> Self {
        Self(count as u64)
    }

    pub fn total(counts: impl Iterator<Item = RunCount>) -> Self {
        Self(counts.map(|count| count.0).sum())
    }
}

impl fmt::Display for RunCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{LeaseId, PrNumber, TaskId};

    #[test]
    fn a_pr_number_takes_one_optional_hash_and_digits() {
        let parsed = ["7", "#7", "##7", "#", "", "#x"].map(|text| text.parse::<PrNumber>().is_ok());
        assert_eq!(parsed, [true, true, false, false, false, false]);
    }

    #[test]
    fn a_counted_id_is_its_prefix_and_canonical_digits() {
        let parsed = ["lease-1", "lease-10", "lease-0", "lease-01", "lease-+1", "lease-", "lease--1", "task-1", " lease-1"]
            .map(|text| text.parse::<LeaseId>().is_ok());
        assert_eq!(parsed, [true, true, true, false, false, false, false, false, false]);
        assert!("task-01".parse::<TaskId>().is_err() && "task-7".parse::<TaskId>().is_ok());
    }
}
