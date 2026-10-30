//! The one place id and newtype text is taken apart: `FromStr` and `TryFrom<String>` for the id
//! and text types. Their fields stay private to `ids`, which this module is part of.
#![allow(clippy::disallowed_methods)]

use super::{AgentName, Branch, CheckName, LeaseId, Malformed, Origin, PrNumber, Reason, Sha, TaskId, Title};
use std::str::FromStr;

macro_rules! counted_parse {
    ($name:ident) => {
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

macro_rules! text_parse {
    ($name:ident) => {
        impl TryFrom<String> for $name {
            type Error = Malformed;

            fn try_from(text: String) -> Result<Self, Malformed> {
                let trimmed = text.trim();
                match trimmed.is_empty() {
                    true => Err(Malformed(format!("{} is empty", Self::WHAT))),
                    false => Ok(Self(trimmed.to_owned())),
                }
            }
        }

        impl FromStr for $name {
            type Err = Malformed;

            fn from_str(text: &str) -> Result<Self, Malformed> {
                Self::try_from(text.to_owned())
            }
        }
    };
}

counted_parse!(TaskId);
counted_parse!(LeaseId);
text_parse!(Title);
text_parse!(AgentName);
text_parse!(Reason);
text_parse!(Sha);
text_parse!(CheckName);
text_parse!(Origin);
text_parse!(Branch);

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
