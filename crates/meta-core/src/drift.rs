use crate::ids::{Branch, Email, PrNumber, Sha};
use crate::ledger::Ledger;
use crate::merges::Unreached;
use crate::set_aside::one_line;
use serde::Deserialize;
use std::collections::BTreeSet;

/// The open PRs as `gh pr list --state open --json number,baseRefName,commits` reports them.
#[derive(Debug)]
pub struct OpenPrs(Vec<Open>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Open {
    number: PrNumber,
    base_ref_name: Branch,
    commits: Vec<Commit>,
}

#[derive(Debug, Deserialize)]
struct Commit {
    oid: Sha,
    authors: Vec<Author>,
}

/// An author as GitHub lists one; the email may be empty, which no identity matches.
#[derive(Debug, Deserialize)]
struct Author {
    email: String,
}

impl Commit {
    /// GitHub lists the commit's author first and its `Co-authored-by` trailers after, so only the
    /// first is the identity the commit was made under.
    fn author(&self) -> Option<&Author> {
        self.authors.first()
    }

    fn is_by(&self, identity: &Email) -> bool {
        self.author().is_some_and(|author| Email::try_from(author.email.clone()).is_ok_and(|email| &email == identity))
    }
}

impl OpenPrs {
    /// The rows the adapter asks for and the most it keeps. GitHub estimates a query at N + 100N + 100*100N
    /// nodes for N PRs (each with up to 100 commits, each with up to 100 authors) and rejects more than
    /// 500,000, which allows 49 PRs (50 gives 505,050). A full page reads as truncated.
    pub const LIMIT: usize = 49;
    /// The most commits `gh` lists per PR; a PR at this count may have more that are not listed.
    pub const COMMIT_LIMIT: usize = 100;

    pub fn from_gh(listed: &str) -> Result<Self, Unreached> {
        match serde_json::from_str::<Vec<Open>>(listed) {
            Err(error) => Err(Unreached::by(&format!("gh pr list open: {error}"))),
            Ok(rows) if rows.len() >= Self::LIMIT => Err(Unreached::by(&"open PR list may be truncated")),
            Ok(rows) => Ok(Self(rows)),
        }
    }
}

impl Ledger {
    /// The drift section of `meta status`: habits no gate enforces, flagged and never blocked.
    pub(crate) fn drift(&self, open: &Result<OpenPrs, Unreached>, identity: &Result<Email, Unreached>) -> String {
        const HEADING: &str = "drift (flagged)\n";
        let open = match open {
            Ok(open) => open,
            Err(why) => return format!("{HEADING}  drift unavailable: {why}\n"),
        };
        let linked: BTreeSet<PrNumber> = self.linked_prs.values().flatten().copied().collect();
        let main = Branch::main();
        let lines: Vec<String> = open
            .0
            .iter()
            .flat_map(|pr| {
                let unlinked = (!linked.contains(&pr.number)).then(|| format!("{}: no linked task", pr.number));
                let based = (pr.base_ref_name != main)
                    .then(|| format!("{}: base is {}, not {main}", pr.number, one_line(&pr.base_ref_name.to_string())));
                let authors: Vec<String> = match identity {
                    Ok(_) if pr.commits.len() >= OpenPrs::COMMIT_LIMIT => {
                        vec![format!("{}: commit authors unavailable: commit list may be truncated", pr.number)]
                    }
                    Ok(identity) => pr
                        .commits
                        .iter()
                        .filter(|commit| !commit.is_by(identity))
                        .map(|commit| {
                            let author = commit.author().map_or("none", |author| author.email.as_str());
                            format!(
                                "{}: commit {} is not by the identity email, author {}",
                                pr.number,
                                commit.oid,
                                one_line(author)
                            )
                        })
                        .collect(),
                    Err(why) => vec![format!("{}: commit authors unavailable: {why}", pr.number)],
                };
                unlinked.into_iter().chain(based).chain(authors)
            })
            .collect();
        let body = match lines.is_empty() {
            true => "  (none)\n".to_owned(),
            false => lines.iter().map(|line| format!("  {line}\n")).collect(),
        };
        format!("{HEADING}{body}")
    }
}
