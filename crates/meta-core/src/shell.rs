use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bypass {
    PrMerge,
    ApiMerge,
    PushMain,
    PushUnnamed,
    Unexpanded,
}

impl fmt::Display for Bypass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PrMerge => "gh pr merge bypasses meta",
            Self::ApiMerge => "a merge through gh api bypasses meta",
            Self::PushMain => "a git push to main bypasses meta",
            Self::PushUnnamed => "name the branch: git push origin <branch>",
            Self::Unexpanded => "a git push or gh api with an unexpanded $ cannot be checked; spell the value out",
        })
    }
}

pub struct ShellCommand(Vec<Vec<String>>);

impl ShellCommand {
    const CONTINUATIONS: [&str; 2] = ["\\\r\n", "\\\n"];
    const SUBSTITUTIONS: [&str; 2] = ["$(", "`"];
    const SEPARATORS: [char; 6] = [';', '&', '|', '\n', '(', ')'];
    const QUOTES: [char; 2] = ['\'', '"'];
    const MERGE_ENDPOINTS: [&str; 2] = ["/merge", "mergePullRequest"];
    const GH_VALUED: [&str; 2] = ["-R", "--repo"];
    const GIT_VALUED: [&str; 2] = ["-C", "-c"];
    const PUSH_VALUED: [&str; 5] = ["-o", "--push-option", "--repo", "--receive-pack", "--exec"];
    const EVERY_BRANCH: [&str; 2] = ["--all", "--mirror"];
    const MAIN: &str = "main";
    const CURRENT: &str = "HEAD";

    pub fn parse(text: &str) -> Self {
        let joined =
            Self::CONTINUATIONS.iter().fold(text.to_owned(), |text, continuation| text.replace(continuation, ""));
        let separated = Self::SUBSTITUTIONS.iter().fold(joined, |text, opener| text.replace(opener, ";"));
        Self(
            separated
                .split(Self::SEPARATORS)
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
        self.0.iter().find_map(|words| Self::gh(words).or_else(|| Self::git(words)))
    }

    fn gh(words: &[String]) -> Option<Bypass> {
        let rest = Self::after(words, "gh")?;
        let positional = Self::positional(rest, &Self::GH_VALUED);
        match positional.as_slice() {
            ["pr", "merge", ..] => Some(Bypass::PrMerge),
            ["api", ..] if rest.iter().any(|word| word.contains('$')) => Some(Bypass::Unexpanded),
            ["api", ..] if rest.iter().any(|word| Self::MERGE_ENDPOINTS.iter().any(|end| word.contains(end))) => {
                Some(Bypass::ApiMerge)
            }
            _ => None,
        }
    }

    fn git(words: &[String]) -> Option<Bypass> {
        let rest = Self::after(words, "git")?;
        let (_, args) = Self::skip_git_options(rest).split_first().filter(|(subcommand, _)| *subcommand == "push")?;
        let positional = Self::positional(args, &Self::PUSH_VALUED);
        let destinations: Vec<&str> = positional.iter().skip(1).map(|refspec| Self::destination(refspec)).collect();
        [
            (rest.iter().any(|word| word.contains('$')), Bypass::Unexpanded),
            (args.iter().any(|arg| Self::EVERY_BRANCH.contains(&arg.as_str())), Bypass::PushMain),
            (destinations.contains(&Self::MAIN), Bypass::PushMain),
            (destinations.is_empty() || destinations.contains(&Self::CURRENT), Bypass::PushUnnamed),
        ]
        .into_iter()
        .find_map(|(found, bypass)| found.then_some(bypass))
    }

    fn after<'a>(words: &'a [String], program: &str) -> Option<&'a [String]> {
        let at = words.iter().position(|word| word.rsplit('/').next() == Some(program))?;
        words.get(at + 1..)
    }

    fn positional<'a>(words: &'a [String], valued: &[&str]) -> Vec<&'a str> {
        let mut skip_value = false;
        words
            .iter()
            .filter(|word| {
                let flag = word.starts_with('-');
                let keep = !flag && !skip_value;
                skip_value = valued.contains(&word.as_str());
                keep
            })
            .map(String::as_str)
            .collect()
    }

    fn skip_git_options(words: &[String]) -> &[String] {
        match words {
            [option, _, rest @ ..] if Self::GIT_VALUED.contains(&option.as_str()) => Self::skip_git_options(rest),
            [option, rest @ ..] if option.starts_with('-') => Self::skip_git_options(rest),
            subcommand => subcommand,
        }
    }

    fn destination(refspec: &str) -> &str {
        let target = refspec.rsplit(':').next().unwrap_or(refspec).trim_start_matches('+');
        target.strip_prefix("refs/heads/").unwrap_or(target)
    }
}
