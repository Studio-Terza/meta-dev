use super::command::run;
use crate::{GhReplies, GitHub, PortError};
use meta_core::{Cleared, Merges, OpenPrs, PrNumber, Pull, Updating};
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub struct GhCli;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Refs {
    head_ref_name: String,
    base_ref_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Head {
    head_ref_name: String,
}

/// The one `gh run list` call, for a branch name already read by `pull` or by `runs`.
fn branch_runs(branch: &str) -> Result<String, PortError> {
    run("gh", &["run", "list", "--branch", branch, "--json", "databaseId", "--limit", "1000"])
}

impl GitHub for GhCli {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        let number = pr.digits();
        let fields = "headRefOid,headRefName,baseRefName,state,mergeStateStatus,url,statusCheckRollup";
        let view = run("gh", &["pr", "view", &number, "--json", fields])?;
        let refs: Refs = serde_json::from_str(&view).map_err(|error| PortError::new(format!("gh pr view: {error}")))?;
        let protection =
            format!("repos/{{owner}}/{{repo}}/branches/{}/protection/required_status_checks", refs.base_ref_name);
        let required = run("gh", &["api", &protection]);
        let branch = refs.head_ref_name;
        let runs = branch_runs(&branch)?;
        GhReplies { view, required, runs }.pull()
    }

    fn runs(&self, pr: PrNumber) -> Result<String, PortError> {
        let view = run("gh", &["pr", "view", &pr.digits(), "--json", "headRefName"])?;
        let head: Head = serde_json::from_str(&view).map_err(|error| PortError::new(format!("gh pr view: {error}")))?;
        branch_runs(&head.head_ref_name)
    }

    fn merged_list(&self) -> Result<String, PortError> {
        let limit = (Merges::LIMIT + 1).to_string();
        run("gh", &["pr", "list", "--state", "merged", "--json", "number", "--limit", &limit])
    }

    fn open_list(&self) -> Result<String, PortError> {
        let limit = OpenPrs::LIMIT.to_string();
        run("gh", &["pr", "list", "--state", "open", "--json", "number,baseRefName,commits", "--limit", &limit])
    }

    fn update_branch(&mut self, updating: &Updating) -> Result<(), PortError> {
        run("gh", &["pr", "update-branch", &updating.pr().digits()]).map(|_| ())
    }

    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError> {
        let number = cleared.pr().digits();
        let head = cleared.head().to_string();
        run("gh", &["pr", "merge", &number, "--squash", "--match-head-commit", &head]).map(|_| ())
    }
}
