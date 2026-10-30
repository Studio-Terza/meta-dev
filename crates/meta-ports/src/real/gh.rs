use super::command::run;
use crate::{GitHub, PortError};
use meta_core::{Check, CheckName, Cleared, Outcome, PrNumber, Pull, RunCount, Sha};
use serde::Deserialize;
use serde::de::IgnoredAny;

#[derive(Debug, Clone, Copy)]
pub struct GhCli;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullView {
    head_ref_oid: Sha,
    head_ref_name: String,
    base_ref_name: String,
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
            Some("PENDING" | "EXPECTED") | None => Outcome::Pending,
            Some(_) => Outcome::Red,
        };
        self.name.or(self.context).map(|name| Check { name, outcome })
    }
}

fn parsed<T: for<'de> Deserialize<'de>>(what: &str, text: &str) -> Result<T, PortError> {
    serde_json::from_str(text).map_err(|error| PortError::new(format!("{what}: {error}")))
}

impl GitHub for GhCli {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        let number = pr.digits();
        let fields = "headRefOid,headRefName,baseRefName,statusCheckRollup";
        let view: PullView = parsed("gh pr view", &run("gh", &["pr", "view", &number, "--json", fields])?)?;
        let protection =
            format!("repos/{{owner}}/{{repo}}/branches/{}/protection/required_status_checks", view.base_ref_name);
        let required = run("gh", &["api", &protection])
            .and_then(|text| parsed::<Required>("required checks", &text))
            .map(|required| required.contexts)
            .unwrap_or_default();
        let runs =
            run("gh", &["run", "list", "--branch", &view.head_ref_name, "--json", "databaseId", "--limit", "1000"])?;
        let runs: Vec<IgnoredAny> = parsed("gh run list", &runs)?;
        Ok(Pull {
            head: view.head_ref_oid,
            required,
            reported: view.status_check_rollup.into_iter().filter_map(Rollup::check).collect(),
            runs: RunCount::of(runs.len()),
        })
    }

    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError> {
        let number = cleared.pr().digits();
        let head = cleared.head().to_string();
        run("gh", &["pr", "merge", &number, "--squash", "--match-head-commit", &head]).map(|_| ())
    }
}
