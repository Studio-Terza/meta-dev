use super::command::run;
use crate::{GhReplies, GitHub, PortError};
use meta_core::{Cleared, Merges, PrNumber, Pull};
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub struct GhCli;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Refs {
    head_ref_name: String,
    base_ref_name: String,
}

impl GitHub for GhCli {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        let number = pr.digits();
        let fields = "headRefOid,headRefName,baseRefName,state,url,statusCheckRollup";
        let view = run("gh", &["pr", "view", &number, "--json", fields])?;
        let refs: Refs = serde_json::from_str(&view).map_err(|error| PortError::new(format!("gh pr view: {error}")))?;
        let protection =
            format!("repos/{{owner}}/{{repo}}/branches/{}/protection/required_status_checks", refs.base_ref_name);
        let required = run("gh", &["api", &protection]);
        let branch = refs.head_ref_name;
        let runs = run("gh", &["run", "list", "--branch", &branch, "--json", "databaseId", "--limit", "1000"])?;
        GhReplies { view, required, runs }.pull()
    }

    fn merged_list(&self) -> Result<String, PortError> {
        let limit = (Merges::LIMIT + 1).to_string();
        run("gh", &["pr", "list", "--state", "merged", "--json", "number", "--limit", &limit])
    }

    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError> {
        let number = cleared.pr().digits();
        let head = cleared.head().to_string();
        run("gh", &["pr", "merge", &number, "--squash", "--match-head-commit", &head]).map(|_| ())
    }
}
