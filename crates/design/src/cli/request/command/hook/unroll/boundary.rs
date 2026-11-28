use std::fmt;
use std::path::{Path, PathBuf};

use console_ports::{Heard, Shown};
use grammar_shell::{Construct, Editor, Flagged};
use newtype::Wrapping;
use serde::{Deserialize, Serialize};

use super::super::brief::Brief;
use super::Refusal;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

#[derive(Debug, Deserialize)]
pub(crate) struct Running {
    #[serde(default)]
    cwd: PathBuf,
    #[serde(flatten)]
    tool: Tool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tool_name", content = "tool_input")]
pub(crate) enum Tool {
    Bash(Input),
    Agent(Brief),
    SendMessage(Brief),
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Input {
    #[serde(default)]
    command: Option<String>,
}

#[derive(Debug, Serialize)]
struct Denial {
    #[serde(rename = "hookSpecificOutput")]
    output: Specific,
}

#[derive(Debug, Serialize)]
struct Specific {
    #[serde(rename = "hookEventName")]
    event: HookEvent,
    #[serde(rename = "permissionDecision")]
    decision: Permission,
    #[serde(rename = "permissionDecisionReason")]
    reason: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Permission {
    #[serde(rename = "deny")]
    Deny,
}

impl Running {
    pub(crate) fn heard(heard: Heard) -> Result<Running, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub(crate) fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub(crate) fn tool(&self) -> &Tool {
        &self.tool
    }
}

impl Input {
    pub(crate) fn command(&self) -> &str {
        self.command.as_deref().unwrap_or_default()
    }
}

impl Refusal {
    pub(crate) fn denied(&self) -> Result<Uttered, Failure> {
        let denial = Denial {
            output: Specific { event: HookEvent::PreToolUse, decision: Permission::Deny, reason: self.to_string() },
        };
        let json = serde_json::to_string(&denial).map_err(Failure::Render)?;
        Ok(Uttered::new(Shown::said(format_args!("{json}\n")), Ok(Outcome::Blocked)))
    }

    const fn word(flagged: Flagged) -> &'static str {
        match flagged {
            Flagged::Rm => "rm",
            Flagged::Rmdir => "rmdir",
            Flagged::Delete => "-delete",
        }
    }

    const fn editor(editor: Editor) -> &'static str {
        match editor {
            Editor::Sed => "sed -i",
            Editor::Perl => "perl -pi",
        }
    }

    const fn construct(construct: Construct) -> &'static str {
        match construct {
            Construct::ShellScript => "a `sh -c` script",
            Construct::Loop => "a loop",
            Construct::Conditional => "an `if` or `case`",
            Construct::Function => "a function",
            Construct::Subshell => "a subshell",
            Construct::Group => "a `{ ...; }` group",
            Construct::Substitution => "a command substitution",
            Construct::Xargs => "`xargs`",
            Construct::Find => "`find`",
            Construct::Heredoc => "a heredoc fed to a shell",
        }
    }

    fn within(within: Option<Construct>) -> String {
        within.map_or_else(String::new, |construct| format!(" inside {}", Refusal::construct(construct)))
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Unroll(nested) => {
                let word = Refusal::word(nested.flagged());
                let construct = Refusal::construct(nested.construct());
                write!(
                    formatter,
                    "Refused: `{word}` inside {construct}. The harness flags `{word}` there and stalls the call. \
                     Unroll it into separate plain top-level commands: list the targets first, \
                     then run each `rm` as its own command with literal paths."
                )
            }
            Refusal::InPlace { editor, within } => write!(
                formatter,
                "Refused: `{}`{} edits a file in place. Edit files only with the Edit or Write tools.",
                Refusal::editor(*editor),
                Refusal::within(*within)
            ),
            Refusal::Wrote { path, repository, within } => write!(
                formatter,
                "Refused: a shell write{} into {}, under the repository at {}. Write files there only with the \
                 Edit or Write tools.",
                Refusal::within(*within),
                path.display(),
                repository.display()
            ),
            Refusal::Moved { path, repository, within } => write!(
                formatter,
                "Refused: `mv`{} takes {} out of the repository at {}, and a move out of a repository is a delete. \
                 Use `git mv` to move it within the repository, or `git rm` to delete it.",
                Refusal::within(*within),
                path.display(),
                repository.display()
            ),
            Refusal::Unresolved { written, within } => write!(
                formatter,
                "Refused: a shell write or move{} onto {} after a `cd` whose target cannot be worked out. \
                 Use absolute paths, or edit files with the Edit or Write tools.",
                Refusal::within(*within),
                written.display()
            ),
        }
    }
}
