//! Deleted by task-5.
#![allow(clippy::disallowed_methods)]

use crate::event::Attempt;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bypass {
    PrMerge,
    ApiWrite,
    PushMain,
    PushUnnamed,
    Unexpanded,
    AgentProgram,
}

impl Bypass {
    pub fn attempt(self) -> Attempt {
        match self {
            Self::AgentProgram => Attempt::AgentStart,
            _ => Attempt::Merge,
        }
    }
}

impl fmt::Display for Bypass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PrMerge => "gh pr merge bypasses meta",
            Self::ApiWrite => "a gh api call that writes bypasses meta",
            Self::PushMain => "a git push to main bypasses meta",
            Self::PushUnnamed => "name the branch: git push origin <branch>",
            Self::Unexpanded => "a git push or gh api with an unexpanded $ cannot be checked; spell the value out",
            Self::AgentProgram => "starting an agent with the claude program bypasses meta; use the Agent tool",
        })
    }
}

pub struct ShellCommand(Vec<Vec<String>>);

impl ShellCommand {
    const CONTINUATIONS: [&str; 2] = ["\\\r\n", "\\\n"];
    const SUBSTITUTIONS: [&str; 2] = ["$(", "`"];
    const SEPARATORS: [char; 6] = [';', '&', '|', '\n', '(', ')'];
    const QUOTES: [char; 2] = ['\'', '"'];
    const WRAPPERS: [&str; 6] = ["env", "nohup", "exec", "command", "time", "sudo"];
    const AGENT_PROGRAM: &str = "claude";
    const GH_VALUED: [&str; 2] = ["-R", "--repo"];
    const API_VALUED: [&str; 15] = [
        "-X",
        "--method",
        "-H",
        "--header",
        "-f",
        "-F",
        "--field",
        "--raw-field",
        "--input",
        "-q",
        "--jq",
        "-t",
        "--template",
        "--hostname",
        "--cache",
    ];
    const API_METHOD: [&str; 2] = ["-X", "--method"];
    const API_BODY: [&str; 5] = ["-f", "-F", "--field", "--raw-field", "--input"];
    const GRAPHQL: &str = "graphql";
    const MUTATION: &str = "mutation";
    const QUERY: [&str; 3] = ["query=", "-fquery=", "-Fquery="];
    const READ: &str = "GET";
    const GIT_VALUED: [&str; 5] = ["-C", "-c", "--git-dir", "--work-tree", "--namespace"];
    const PUSH_VALUED: [&str; 9] =
        ["-o", "--push-option", "--repo", "--receive-pack", "--exec", "-P", "--prefix", "-m", "--message"];
    const EVERY_BRANCH: [&str; 2] = ["--all", "--mirror"];
    const MAIN: &str = "main";
    const CURRENT: [&str; 2] = ["HEAD", "@"];

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
        self.0.iter().find_map(|words| Self::agent(words).or_else(|| Self::gh(words)).or_else(|| Self::git(words)))
    }

    fn agent(words: &[String]) -> Option<Bypass> {
        let program = words.iter().find(|word| !word.contains('=') && !Self::WRAPPERS.contains(&word.as_str()))?;
        (program.rsplit('/').next() == Some(Self::AGENT_PROGRAM)).then_some(Bypass::AgentProgram)
    }

    fn gh(words: &[String]) -> Option<Bypass> {
        let rest = Self::after(words, "gh")?;
        match Self::positional(rest, &Self::GH_VALUED).as_slice() {
            ["pr", "merge", ..] => Some(Bypass::PrMerge),
            ["api", ..] => {
                let args = Self::after(rest, "api")?;
                match Self::positional(args, &Self::API_VALUED).first() {
                    Some(&Self::GRAPHQL) => Self::graphql(args),
                    _ => Self::rest_api(args),
                }
            }
            _ => None,
        }
    }

    fn graphql(args: &[String]) -> Option<Bypass> {
        let query = args.iter().find_map(|word| {
            Self::QUERY
                .iter()
                .find_map(|prefix| word.strip_prefix(prefix))
                .map(|query| query.trim_matches(Self::QUOTES))
        });
        let unreadable = query.is_none_or(|query| query.is_empty() || query.starts_with(['$', '@']));
        let writes =
            args.iter().any(|word| word.to_lowercase().contains(Self::MUTATION) || word.starts_with("--input"));
        (unreadable || writes).then_some(Bypass::ApiWrite)
    }

    fn rest_api(args: &[String]) -> Option<Bypass> {
        let explicit = args.iter().enumerate().find_map(|(at, word)| match word.as_str() {
            flag if Self::API_METHOD.contains(&flag) => args.get(at + 1).map(String::as_str),
            joined => joined.strip_prefix("--method=").or_else(|| joined.strip_prefix("-X").filter(|m| !m.is_empty())),
        });
        let body = args.iter().any(|word| {
            Self::API_BODY.iter().any(|flag| word == flag || word.starts_with(&format!("{flag}=")))
                || ((word.starts_with("-f") || word.starts_with("-F")) && !word.starts_with("--"))
        });
        let method = explicit.unwrap_or(if body { "POST" } else { Self::READ });
        [
            (args.iter().any(|word| word.contains('$')), Bypass::Unexpanded),
            (!method.eq_ignore_ascii_case(Self::READ), Bypass::ApiWrite),
        ]
        .into_iter()
        .find_map(|(found, bypass)| found.then_some(bypass))
    }

    fn git(words: &[String]) -> Option<Bypass> {
        let rest = Self::after(words, "git")?;
        let args = match Self::skip_git_options(rest) {
            [push, args @ ..] if push == "push" => args,
            [subtree, push, args @ ..] if subtree == "subtree" && push == "push" => args,
            _ => return None,
        };
        let positional = Self::positional(args, &Self::PUSH_VALUED);
        let destinations: Vec<&str> = positional.iter().skip(1).map(|refspec| Self::destination(refspec)).collect();
        [
            (rest.iter().any(|word| word.contains('$')), Bypass::Unexpanded),
            (args.iter().any(|arg| Self::EVERY_BRANCH.contains(&arg.as_str())), Bypass::PushMain),
            (destinations.contains(&Self::MAIN), Bypass::PushMain),
            (
                destinations.is_empty() || destinations.iter().any(|branch| Self::CURRENT.contains(branch)),
                Bypass::PushUnnamed,
            ),
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
                let keep = !word.starts_with('-') && !skip_value;
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
