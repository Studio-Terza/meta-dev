use std::fmt;

use design_model::{FeatureName, Verdict};
use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};
use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure, Written};

use super::{Judged, Judgments};
use crate::IdealId;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineNumber(u32);

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked, strategy = "[a-z][a-z0-9_/.-]{0,30}")]
pub struct SourcePath(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[0-9a-f]{40}")]
pub struct Revision(String);

#[derive(Debug, thiserror::Error)]
#[error("a revision or source path is never blank")]
pub struct Blank;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    file: SourcePath,
    line: LineNumber,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ruling {
    ideal: IdealId,
    verdict: Verdict,
    cited: Option<Evidence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Judgment {
    feature: FeatureName,
    ruling: Ruling,
    revision: Revision,
}

#[derive(Debug, Columns)]
struct JudgmentRow {
    feature: FeatureName,
    ideal: IdealId,
    verdict: Verdict,
    file: Option<SourcePath>,
    line: Option<LineNumber>,
    revision: Revision,
}

impl Checked for SourcePath {
    type Error = Blank;

    fn checked(text: &String) -> Result<&String, Blank> {
        text.chars().next().map(|_first| text).ok_or(Blank)
    }
}

impl Checked for Revision {
    type Error = Blank;

    fn checked(text: &String) -> Result<&String, Blank> {
        text.chars().next().map(|_first| text).ok_or(Blank)
    }
}

impl Evidence {
    #[must_use]
    pub fn new(file: SourcePath, line: LineNumber) -> Evidence {
        Evidence { file, line }
    }
}

impl Ruling {
    #[must_use]
    pub fn new(ideal: IdealId, verdict: Verdict, cited: Option<Evidence>) -> Ruling {
        Ruling { ideal, verdict, cited }
    }

    #[must_use]
    pub fn verdict(&self) -> Verdict {
        self.verdict
    }

    #[must_use]
    pub fn judged(self, feature: FeatureName, revision: Revision) -> Judgment {
        Judgment { feature, ruling: self, revision }
    }
}

impl Judgment {
    fn inserted(&self) -> Prepared<()> {
        let Judgment { feature, ruling: Ruling { ideal, verdict, cited }, revision } = self;
        let file = cited.as_ref().map(|found| found.file.clone());
        let line = cited.as_ref().map(|found| found.line);
        store_ports::query!(
            r#"
INSERT INTO
  judgments (feature, ideal, verdict, file, line, revision)
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6)
"#,
            feature: FeatureName,
            ideal: IdealId,
            verdict: Verdict,
            file: Option<SourcePath>,
            line: Option<LineNumber>,
            revision: Revision
        )
    }
}

impl From<JudgmentRow> for Judgment {
    fn from(row: JudgmentRow) -> Judgment {
        let JudgmentRow { feature, ideal, verdict, file, line, revision } = row;
        let cited = file.zip(line).map(|(path, number)| Evidence { file: path, line: number });
        Judgment { feature, ruling: Ruling { ideal, verdict, cited }, revision }
    }
}

impl Batch for Judged {
    type Failure = StoreFailure;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let Judged(judgments) = self;
        statements.written_all(judgments.iter().map(Judgment::inserted))
    }
}

impl Query for Judgments {
    type Rows = Vec<Judgment>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Judgment>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            JudgmentRow,
            r#"
SELECT
  feature,
  ideal,
  verdict,
  file,
  line,
  revision
FROM
  judgments
ORDER BY
  id
"#,
        ))?;
        Ok(rows.into_iter().map(Judgment::from).collect())
    }
}

impl fmt::Display for Evidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.file, self.line)
    }
}

impl fmt::Display for Judgment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Ruling { ideal, verdict, cited } = &self.ruling;
        let shown = cited.as_ref().map_or_else(|| String::from("-"), ToString::to_string);
        write!(formatter, "{}\ti{ideal}\t{verdict}\t{shown}", self.feature)
    }
}
