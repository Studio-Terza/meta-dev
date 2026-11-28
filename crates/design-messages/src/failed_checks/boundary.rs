use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use hex::FromHex;
use newtype::{Checked, CheckedDeserialize, Display, Newtype, Serialize, Spelled};
use port_core::Op;
use serde::Deserialize;
use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure, Written};

#[derive(Clone, Copy, Debug, Op)]
pub struct FailedBefore {
    pull: PullNumber,
    head: CommitSha,
}

#[derive(Debug, Op)]
pub struct Failing {
    pull: PullNumber,
    head: CommitSha,
    checks: Vec<CheckName>,
}

impl FailedBefore {
    #[must_use]
    pub fn new(pull: PullNumber, head: CommitSha) -> FailedBefore {
        FailedBefore { pull, head }
    }
}

impl Failing {
    #[must_use]
    pub fn new(pull: PullNumber, head: CommitSha, checks: Vec<CheckName>) -> Failing {
        Failing { pull, head, checks }
    }
}

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PullNumber(NonZeroU32);

#[derive(Spelled, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct CommitSha([u8; 20]);

#[derive(Debug, thiserror::Error)]
#[error("not a full commit sha of forty hexadecimal digits")]
pub struct Unhexed;

#[derive(Newtype, Display, CheckedDeserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked)]
pub struct CheckName(String);

#[derive(Debug, thiserror::Error)]
#[error("a check name is blank or holds a control character")]
pub struct Unnamed;

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
struct FailedCheckRow {
    name: CheckName,
}

impl Checked for CheckName {
    type Error = Unnamed;

    fn checked(text: &String) -> Result<&String, Unnamed> {
        let named = text.chars().next().is_some() && text.chars().all(|letter| !letter.is_control());
        named.then_some(text).ok_or(Unnamed)
    }
}

impl FromStr for CommitSha {
    type Err = Unhexed;

    fn from_str(text: &str) -> Result<CommitSha, Unhexed> {
        <[u8; 20]>::from_hex(text).map(CommitSha).map_err(|_unhexed| Unhexed)
    }
}

impl TryFrom<String> for CommitSha {
    type Error = Unhexed;

    fn try_from(text: String) -> Result<CommitSha, Unhexed> {
        text.parse()
    }
}

impl fmt::Display for CommitSha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(self.0))
    }
}

impl FailedBefore {
    fn cleared(pull: PullNumber) -> Prepared<()> {
        store_ports::query!(
            r#"
DELETE FROM failed_checks
WHERE
  pull = ?1
"#,
            pull: PullNumber
        )
    }

    fn inserted(pull: PullNumber, head: CommitSha, name: &CheckName) -> Prepared<()> {
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO failed_checks (pull, head, name)
VALUES
  (?1, ?2, ?3)
"#,
            pull: PullNumber,
            head: CommitSha,
            name: CheckName
        )
    }
}

impl Batch for Failing {
    type Failure = StoreFailure;

    type Done = Written;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let Failing { pull, head, checks } = self;
        let rows = [FailedBefore::cleared(pull)]
            .into_iter()
            .chain(checks.iter().map(|name| FailedBefore::inserted(pull, head, name)));
        statements.written(rows)
    }
}

impl Query for FailedBefore {
    type Rows = Vec<CheckName>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<CheckName>, StoreFailure> {
        let FailedBefore { pull, head } = self;
        let rows = statements.rows(store_ports::query!(
            FailedCheckRow,
            r#"
SELECT
  name
FROM
  failed_checks
WHERE
  pull = ?1
  AND head = ?2
ORDER BY
  name
"#,
            pull: PullNumber,
            head: CommitSha
        ))?;
        Ok(rows.into_iter().map(|row| row.name).collect())
    }
}
