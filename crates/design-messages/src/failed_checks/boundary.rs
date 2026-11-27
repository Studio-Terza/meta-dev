use std::num::NonZeroU32;

use newtype::{Checked, CheckedDeserialize, Display, Newtype};
use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure};

use super::{FailedBefore, Failing};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PullNumber(NonZeroU32);

#[derive(Newtype, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    fn inserted(pull: PullNumber, name: &CheckName) -> Prepared<()> {
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO failed_checks (pull, name)
VALUES
  (?1, ?2)
"#,
            pull: PullNumber,
            name: CheckName
        )
    }
}

impl Batch for Failing {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let Failing { pull, checks } = self;
        let rows = [FailedBefore::cleared(pull)]
            .into_iter()
            .chain(checks.iter().map(|name| FailedBefore::inserted(pull, name)));
        statements.written_all(rows)
    }
}

impl Query for FailedBefore {
    type Rows = Vec<CheckName>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<CheckName>, StoreFailure> {
        let FailedBefore(pull) = self;
        let rows = statements.rows(store_ports::query!(
            FailedCheckRow,
            r#"
SELECT
  name
FROM
  failed_checks
WHERE
  pull = ?1
ORDER BY
  name
"#,
            pull: PullNumber
        ))?;
        Ok(rows.into_iter().map(|row| row.name).collect())
    }
}
