use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bypass {
    PrMerge,
    ApiMerge,
    PushMain,
}

impl fmt::Display for Bypass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PrMerge => "gh pr merge",
            Self::ApiMerge => "a merge through gh api",
            Self::PushMain => "a git push to main",
        })
    }
}

pub struct ShellCommand(Vec<Vec<String>>);

impl ShellCommand {
    const SEPARATORS: [char; 8] = [';', '&', '|', '\n', '(', ')', '`', '$'];
    const QUOTES: [char; 2] = ['\'', '"'];
    const PR_MERGE: [&str; 3] = ["gh", "pr", "merge"];
    const MERGE_ENDPOINTS: [&str; 2] = ["/merge", "mergePullRequest"];
    const VALUED_GIT_OPTIONS: [&str; 2] = ["-C", "-c"];
    const MAIN: &str = "main";
    const EVERY_BRANCH: [&str; 2] = ["--all", "--mirror"];

    pub fn parse(text: &str) -> Self {
        Self(
            text.split(Self::SEPARATORS)
                .map(|segment| {
                    segment
                        .split_whitespace()
                        .map(|word| word.trim_matches(Self::QUOTES).to_owned())
                        .filter(|word| !word.is_empty())
                        .collect()
                })
                .collect(),
        )
    }

    pub fn bypass(&self) -> Option<Bypass> {
        self.0.iter().find_map(|words| Self::segment_bypass(words))
    }

    fn segment_bypass(words: &[String]) -> Option<Bypass> {
        let pr_merge = words.windows(Self::PR_MERGE.len()).any(|window| window == Self::PR_MERGE);
        let api_merge = Self::after(words, "gh")
            .and_then(|rest| Self::after(rest, "api"))
            .is_some_and(|rest| rest.iter().any(|word| Self::MERGE_ENDPOINTS.iter().any(|end| word.contains(end))));
        let push_main = Self::after(words, "git")
            .map(Self::skip_git_options)
            .and_then(|rest| rest.split_first())
            .is_some_and(|(subcommand, args)| subcommand == "push" && args.iter().any(|arg| Self::reaches_main(arg)));
        [(pr_merge, Bypass::PrMerge), (api_merge, Bypass::ApiMerge), (push_main, Bypass::PushMain)]
            .into_iter()
            .find_map(|(found, bypass)| found.then_some(bypass))
    }

    fn after<'a>(words: &'a [String], program: &str) -> Option<&'a [String]> {
        let at = words.iter().position(|word| word.rsplit('/').next() == Some(program))?;
        words.get(at + 1..)
    }

    fn skip_git_options(words: &[String]) -> &[String] {
        match words {
            [option, _, rest @ ..] if Self::VALUED_GIT_OPTIONS.contains(&option.as_str()) => {
                Self::skip_git_options(rest)
            }
            [option, rest @ ..] if option.starts_with('-') => Self::skip_git_options(rest),
            subcommand => subcommand,
        }
    }

    fn reaches_main(arg: &str) -> bool {
        let target = arg.rsplit(':').next().unwrap_or(arg);
        let branch = target.trim_start_matches('+');
        let branch = branch.strip_prefix("refs/heads/").unwrap_or(branch);
        branch == Self::MAIN || Self::EVERY_BRANCH.contains(&arg)
    }
}
