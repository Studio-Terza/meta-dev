use crate::ids::{Branch, CheckName, Malformed, RunCount, Sha};
use serde::Deserialize;
use serde::de::{DeserializeOwned, IgnoredAny};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pull {
    head: Sha,
    branch: Branch,
    base: Branch,
    state: PullState,
    merge_state: MergeState,
    required: Vec<CheckName>,
    reported: Vec<Check>,
    runs: RunCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PullState {
    Open,
    Merged,
    Closed,
}

/// What GitHub says about merging the PR into its base: `Behind` when the base has commits the
/// branch lacks, which a branch that must be up to date cannot merge past. Any other report is not
/// acted on here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum MergeState {
    Behind,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Check {
    name: CheckName,
    outcome: Outcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    Green,
    Pending,
    Red,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct View {
    head_ref_oid: Sha,
    head_ref_name: Branch,
    base_ref_name: Branch,
    state: PullState,
    merge_state_status: MergeState,
    status_check_rollup: Vec<Rollup>,
}

#[derive(Deserialize)]
struct Rollup {
    name: Option<CheckName>,
    context: Option<CheckName>,
    status: Option<String>,
    conclusion: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct Required {
    contexts: Vec<CheckName>,
}

impl Rollup {
    fn check(self) -> Option<Check> {
        let finished = self.status.as_deref().is_none_or(|status| status == "COMPLETED");
        let verdict = self.conclusion.or(self.state).filter(|_| finished);
        let outcome = match verdict.as_deref() {
            Some("SUCCESS" | "NEUTRAL" | "SKIPPED") => Outcome::Green,
            Some("PENDING" | "EXPECTED" | "") | None => Outcome::Pending,
            Some(_) => Outcome::Red,
        };
        self.name.or(self.context).map(|name| Check { name, outcome })
    }
}

fn parsed<T: DeserializeOwned>(what: &str, text: &str) -> Result<T, Malformed> {
    serde_json::from_str(text).map_err(|error| Malformed::because(format!("{what}: {error}")))
}

impl Pull {
    pub fn from_gh(view: &str, required: &str, runs: &str) -> Result<Self, Malformed> {
        let view: View = parsed("gh pr view", view)?;
        let required: Required = parsed("required status checks", required)?;
        let runs: Vec<IgnoredAny> = parsed("gh run list", runs)?;
        Ok(Self {
            head: view.head_ref_oid,
            branch: view.head_ref_name,
            base: view.base_ref_name,
            state: view.state,
            merge_state: view.merge_state_status,
            required: required.contexts,
            reported: view.status_check_rollup.into_iter().filter_map(Rollup::check).collect(),
            runs: RunCount::of(runs.len()),
        })
    }

    pub fn is_at(&self, sha: &Sha) -> bool {
        self.head == *sha
    }

    pub(crate) fn head(&self) -> &Sha {
        &self.head
    }

    pub(crate) fn branch(&self) -> &Branch {
        &self.branch
    }

    pub(crate) fn base(&self) -> &Branch {
        &self.base
    }

    pub(crate) fn state(&self) -> PullState {
        self.state
    }

    pub(crate) fn is_behind(&self) -> bool {
        self.merge_state == MergeState::Behind
    }

    pub(crate) fn required(&self) -> &[CheckName] {
        &self.required
    }

    pub(crate) fn runs(&self) -> RunCount {
        self.runs
    }

    pub(crate) fn outcome(&self, name: &CheckName) -> Option<Outcome> {
        self.reported.iter().filter(|check| check.name == *name).map(|check| check.outcome).max()
    }
}
