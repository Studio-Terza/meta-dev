use std::fmt;

use console_ports::{Heard, Shown};
use grammar::{Construct, Flagged, Nested};
use newtype::Wrapping;
use serde::{Deserialize, Serialize};

use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

#[derive(Debug, Deserialize)]
pub struct Running {
    #[serde(default)]
    tool_input: Input,
}

#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    command: Scripted,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(from = "Option<String>")]
enum Scripted {
    Nesting(Nested),
    #[default]
    Unrolled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Unroll(Nested);

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
    pub fn heard(heard: Heard) -> Result<Running, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn answered(&self) -> Result<Uttered, Failure> {
        match self.tool_input.command {
            Scripted::Nesting(nested) => Unroll(nested).denied(),
            Scripted::Unrolled => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PreToolUse)))),
        }
    }
}

impl From<Option<String>> for Scripted {
    fn from(command: Option<String>) -> Scripted {
        command.as_deref().and_then(Nested::read).map_or(Scripted::Unrolled, Scripted::Nesting)
    }
}

impl Unroll {
    fn denied(self) -> Result<Uttered, Failure> {
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
}

impl fmt::Display for Unroll {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = Unroll::word(self.0.flagged());
        let construct = Unroll::construct(self.0.construct());
        write!(
            formatter,
            "Refused: `{word}` inside {construct}. The harness flags `{word}` there and stalls the call. \
             Unroll it into separate plain top-level commands: list the targets first, \
             then run each `rm` as its own command with literal paths."
        )
    }
}
