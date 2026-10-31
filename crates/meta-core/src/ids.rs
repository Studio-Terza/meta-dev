//! Id and newtype text is parsed here: a registered boundary (self/parse-boundaries.toml).
#![allow(clippy::disallowed_methods)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformed(String);

impl Malformed {
    pub fn because(why: impl Into<String>) -> Self {
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
                    .filter(|id| id.to_string() == text)
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
counted_id!(EscapeId, "escape-");

impl EscapeId {
    /// The position of this escape among the log's `Escaped` records, from 0.
    pub fn index(&self) -> Option<usize> {
        usize::try_from(self.0).ok().and_then(|number| number.checked_sub(1))
    }
}

text!(Title, "title");
text!(AgentName, "agent name");
text!(Reason, "reason");
text!(Sha, "sha");
text!(CheckName, "check name");
text!(Origin, "origin");
text!(Branch, "branch");
text!(EscapeKind, "escape kind");
text!(AgentId, "agent id");
text!(PredictionId, "prediction id");
text!(Observed, "observation");

impl EscapeKind {
    /// The kind of an escape logged before kinds existed: its free-text reason is all it has.
    pub fn unclassified() -> Self {
        Self("unclassified".to_owned())
    }

    /// The kind of an escape that records an agent start without a lease (P1).
    pub fn outside_start() -> Self {
        Self("outside-start".to_owned())
    }
}

/// A reviewer's verdict on a linked PR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Fail,
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        })
    }
}

impl FromStr for Verdict {
    type Err = Malformed;

    fn from_str(text: &str) -> Result<Self, Malformed> {
        match text {
            "pass" => Ok(Self::Pass),
            "fail" => Ok(Self::Fail),
            _ => Err(Malformed(format!("{text:?} is not a verdict: pass or fail"))),
        }
    }
}

/// Who claimed a task: an agent by name, or the lead working it itself (no name, no lease).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "Option<AgentName>", into = "Option<AgentName>")]
pub enum Claimant {
    #[default]
    Lead,
    Agent(AgentName),
}

impl Claimant {
    pub fn is_lead(&self) -> bool {
        matches!(self, Self::Lead)
    }
}

impl From<Option<AgentName>> for Claimant {
    fn from(agent: Option<AgentName>) -> Self {
        agent.map_or(Self::Lead, Self::Agent)
    }
}

impl From<Claimant> for Option<AgentName> {
    fn from(claimant: Claimant) -> Self {
        match claimant {
            Claimant::Lead => None,
            Claimant::Agent(agent) => Some(agent),
        }
    }
}

impl fmt::Display for Claimant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lead => formatter.write_str("the lead (self-claim)"),
            Self::Agent(agent) => write!(formatter, "{agent}"),
        }
    }
}

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
        text.strip_prefix('#')
            .unwrap_or(text)
            .parse()
            .map(Self)
            .map_err(|_| Malformed(format!("{text:?} is not a PR number")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn from_unix_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub fn advanced(self, seconds: u64) -> Self {
        Self(self.0.saturating_add(seconds))
    }
}

impl Timestamp {
    /// This time moved later by `span`.
    pub fn plus(self, span: Span) -> Self {
        self.advanced(span.0)
    }

    /// The span from this time to `later`; none when `later` is not later.
    pub fn span_to(self, later: Self) -> Span {
        Span(later.0.saturating_sub(self.0))
    }
}

/// A stretch of time, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span(u64);

impl Span {
    pub const fn of_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn whole_seconds(self) -> u64 {
        self.0
    }
}

/// A whole number of minutes, at least one: how long a lease runs before its deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u64")]
pub struct Minutes(u64);

impl Minutes {
    const SECONDS: u64 = 60;

    pub fn span(self) -> Span {
        Span(self.0.saturating_mul(Self::SECONDS))
    }
}

impl TryFrom<u64> for Minutes {
    type Error = Malformed;

    fn try_from(count: u64) -> Result<Self, Malformed> {
        match count {
            0 => Err(Malformed("a deadline is at least 1 minute".to_owned())),
            _ => Ok(Self(count)),
        }
    }
}

impl FromStr for Minutes {
    type Err = Malformed;

    fn from_str(text: &str) -> Result<Self, Malformed> {
        text.parse::<u64>()
            .ok()
            .filter(|count| count.to_string() == text)
            .ok_or_else(|| Malformed(format!("{text:?} is not a whole number of minutes")))
            .and_then(Self::try_from)
    }
}

impl fmt::Display for Minutes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} min", self.0)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unix {}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunCount(u64);

impl RunCount {
    pub fn of(count: usize) -> Self {
        Self(count as u64)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn total(counts: impl Iterator<Item = RunCount>) -> Self {
        Self(counts.map(|count| count.0).sum())
    }

    /// The runs in a `gh run list --json databaseId` reply.
    pub fn from_gh(runs: &str) -> Result<Self, Malformed> {
        serde_json::from_str::<Vec<serde::de::IgnoredAny>>(runs)
            .map(|rows| Self::of(rows.len()))
            .map_err(|error| Malformed::because(format!("gh run list: {error}")))
    }
}

/// Review rounds that ended in a `fail` verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FailedRounds(usize);

impl FailedRounds {
    pub fn of(count: usize) -> Self {
        Self(count)
    }

    pub(crate) fn value(self) -> usize {
        self.0
    }
}

impl fmt::Display for FailedRounds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for RunCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{LeaseId, Minutes, PrNumber, TaskId};

    #[test]
    fn a_pr_number_takes_one_optional_hash_and_digits() {
        let parsed = ["7", "#7", "##7", "#", "", "#x"].map(|text| text.parse::<PrNumber>().is_ok());
        assert_eq!(parsed, [true, true, false, false, false, false]);
    }

    #[test]
    fn a_counted_id_is_its_prefix_and_canonical_digits() {
        let parsed =
            ["lease-1", "lease-10", "lease-0", "lease-01", "lease-+1", "lease-", "lease--1", "task-1", " lease-1"]
                .map(|text| text.parse::<LeaseId>().is_ok());
        assert_eq!(parsed, [true, true, true, false, false, false, false, false, false]);
        assert!("task-01".parse::<TaskId>().is_err() && "task-7".parse::<TaskId>().is_ok());
    }

    #[test]
    fn minutes_are_canonical_whole_numbers_of_at_least_one() {
        let parsed = ["1", "90", "0", "01", "+5", "1.5", "-1", "", " 5"].map(|text| text.parse::<Minutes>().is_ok());
        assert_eq!(parsed, [true, true, false, false, false, false, false, false, false]);
    }
}
