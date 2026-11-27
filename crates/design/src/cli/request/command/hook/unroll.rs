use std::fmt;
use std::path::{Component, Path, PathBuf};

use console_ports::{Heard, Shown};
use file_ports::{Files, Kind};
use grammar::{Construct, Directory, Edit, Edited, Editor, Flagged, Nested};
use newtype::Wrapping;
use serde::{Deserialize, Serialize};
use store_ports::Store;

use super::super::Ground;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

#[derive(Debug, Deserialize)]
pub(crate) struct Running {
    #[serde(default)]
    cwd: PathBuf,
    #[serde(default)]
    tool_input: Input,
}

#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    command: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Refusal {
    Unroll(Nested),
    InPlace { editor: Editor, within: Option<Construct> },
    Wrote { path: PathBuf, repository: PathBuf, within: Option<Construct> },
    Moved { path: PathBuf, repository: PathBuf, within: Option<Construct> },
    Unresolved { written: String, within: Option<Construct> },
}

#[derive(Debug)]
struct Judging<'ground, 'run, F: Files, S: Store> {
    ground: &'ground Ground<'run, F, S>,
    cwd: &'ground Path,
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

    pub(crate) fn answered<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>) -> Result<Uttered, Failure> {
        let command = self.tool_input.command.as_deref().unwrap_or_default();
        let judging = Judging { ground, cwd: &self.cwd };
        let refused = Nested::read(command)
            .map(Refusal::Unroll)
            .or_else(|| Edited::read(command).iter().find_map(|edited| judging.refusal(edited)));
        refused
            .map_or_else(|| Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PreToolUse)))), |refusal| refusal.denied())
    }
}

impl<F: Files, S: Store> Judging<'_, '_, F, S> {
    const REPOSITORY: &'static str = ".git";

    fn refusal(&self, edited: &Edited) -> Option<Refusal> {
        let within = edited.construct();
        let directory = edited.directory();
        let unresolved = |written: &str| Refusal::Unresolved { written: written.to_owned(), within };
        match edited.edit() {
            Edit::InPlace(editor) => Some(Refusal::InPlace { editor: *editor, within }),
            Edit::Wrote(written) => match self.resolved(written, directory) {
                Some(path) => self.repository(&path).map(|repository| Refusal::Wrote { path, repository, within }),
                None => Some(unresolved(written)),
            },
            Edit::Moved { sources, destination } => {
                let blind = sources
                    .iter()
                    .chain([destination])
                    .find(|written| self.resolved(written, directory).is_none())
                    .map(|written| unresolved(written));
                let landing = self.resolved(destination, directory).and_then(|path| self.repository(&path));
                blind.or_else(|| {
                    sources.iter().filter_map(|source| self.resolved(source, directory)).find_map(|path| {
                        let repository = self.repository(&path).filter(|held| landing.as_ref() != Some(held))?;
                        Some(Refusal::Moved { path, repository, within })
                    })
                })
            }
        }
    }

    fn resolved(&self, written: &str, directory: &Directory) -> Option<PathBuf> {
        let base = match directory {
            Directory::Started => Some(self.cwd.to_path_buf()),
            Directory::Changed(steps) => Some(steps.iter().fold(self.cwd.to_path_buf(), |base, step| base.join(step))),
            Directory::Unknown => Path::new(written).is_absolute().then(PathBuf::new),
        }?;
        Some(Self::folded(&base.join(written)))
    }

    fn folded(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |resolved, component| match component {
            Component::ParentDir => resolved.parent().map(Path::to_path_buf).unwrap_or(resolved),
            Component::CurDir => resolved,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => resolved.join(component),
        })
    }

    fn repository(&self, path: &Path) -> Option<PathBuf> {
        path.ancestors()
            .find(|directory| self.ground.kind(&directory.join(Self::REPOSITORY)) != Kind::Missing)
            .map(Path::to_path_buf)
    }
}

impl Refusal {
    fn denied(&self) -> Result<Uttered, Failure> {
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
                "Refused: a shell write or move{} onto {written} after a `cd` whose target cannot be worked out. \
                 Use absolute paths, or edit files with the Edit or Write tools.",
                Refusal::within(*within)
            ),
        }
    }
}
