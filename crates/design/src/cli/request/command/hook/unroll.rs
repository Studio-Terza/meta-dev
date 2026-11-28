use std::fmt;
use std::path::{Component, Path, PathBuf};

use console_ports::{Heard, Shown};
use dispatch_ports::{Clock, Shell};
use file_ports::{Files, Kind};
use grammar_shell::{Deletion, Directory, Edit, Edited, Editor, Plain, Unreadable};
use newtype::Wrapping;
use serde::{Deserialize, Serialize};
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::brief::Brief;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};

#[derive(Debug, Deserialize)]
pub struct Running {
    #[serde(default)]
    cwd: PathBuf,
    #[serde(flatten)]
    tool: Tool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tool_name", content = "tool_input")]
enum Tool {
    Bash(Input),
    Agent(Brief),
    SendMessage(Brief),
}

#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    command: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Refusal {
    Unreadable(Unreadable),
    Unruled(Deletion),
    InPlace(Editor),
    Wrote { path: PathBuf, repository: PathBuf },
    Moved { path: PathBuf, repository: PathBuf },
    Unresolved { written: String },
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
    pub fn heard(heard: Heard) -> Result<Running, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn answered<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        match &self.tool {
            Tool::Bash(input) => self.judged(input, ground),
            Tool::Agent(brief) | Tool::SendMessage(brief) => brief.told(ground, crew),
        }
    }

    fn judged<F: Files, S: Store>(&self, input: &Input, ground: &Ground<'_, F, S>) -> Result<Uttered, Failure> {
        let command = input.command.as_deref().unwrap_or_default();
        let judging = Judging { ground, cwd: &self.cwd };
        let refused = match Plain::read(command) {
            Ok(plain) => plain
                .unruled()
                .first()
                .map(|deletion| Refusal::Unruled(*deletion))
                .or_else(|| plain.edits().iter().find_map(|edited| judging.refusal(edited))),
            Err(unreadable) => Some(Refusal::Unreadable(unreadable)),
        };
        refused
            .map_or_else(|| Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PreToolUse)))), |refusal| refusal.denied())
    }
}

impl<F: Files, S: Store> Judging<'_, '_, F, S> {
    const REPOSITORY: &'static str = ".git";

    fn refusal(&self, edited: &Edited) -> Option<Refusal> {
        let directory = edited.directory();
        let unresolved = |written: &str| Refusal::Unresolved { written: written.to_owned() };
        match edited.edit() {
            Edit::InPlace(editor) => Some(Refusal::InPlace(*editor)),
            Edit::Wrote(written) => match self.resolved(written, directory) {
                Some(path) => self.repository(&path).map(|repository| Refusal::Wrote { path, repository }),
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
                        Some(Refusal::Moved { path, repository })
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

    const fn word(deletion: Deletion) -> &'static str {
        match deletion {
            Deletion::Rm => "rm",
            Deletion::Rmdir => "rmdir",
            Deletion::FindDelete => "find -delete",
            Deletion::CargoClean => "cargo clean",
            Deletion::GitClean => "git clean",
        }
    }

    const fn editor(editor: Editor) -> &'static str {
        match editor {
            Editor::Sed => "sed -i",
            Editor::Perl => "perl -pi",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Unreadable(unreadable) => write!(
                formatter,
                "Refused: this is not a plain command, so the harness cannot read it and would stall waiting for \
                 permission (it stopped at `{}`). Run plain commands only: words, literal arguments, quotes, \
                 $NAME, pipes and ; && ||, with no loops, subshells, command substitutions, heredocs or arrays. \
                 Unroll the work into separate plain calls.",
                unreadable.span()
            ),
            Refusal::Unruled(deletion) => {
                let word = Refusal::word(*deletion);
                write!(
                    formatter,
                    "Refused: `{word}` here is run through another command or on arguments that are not literal, so \
                     the harness cannot rule it out and stalls the call. Unroll it: list the targets first, then run \
                     each `{word}` as its own plain top-level command with literal paths."
                )
            }
            Refusal::InPlace(editor) => write!(
                formatter,
                "Refused: `{}` edits a file in place. Edit files only with the Edit or Write tools.",
                Refusal::editor(*editor)
            ),
            Refusal::Wrote { path, repository } => write!(
                formatter,
                "Refused: a shell write into {}, under the repository at {}. Write files there only with the Edit \
                 or Write tools.",
                path.display(),
                repository.display()
            ),
            Refusal::Moved { path, repository } => write!(
                formatter,
                "Refused: `mv` takes {} out of the repository at {}, and a move out of a repository is a delete. \
                 Use `git mv` to move it within the repository, or `git rm` to delete it.",
                path.display(),
                repository.display()
            ),
            Refusal::Unresolved { written } => write!(
                formatter,
                "Refused: a shell write or move onto {written} after a `cd` whose target cannot be worked out. Use \
                 absolute paths, or edit files with the Edit or Write tools."
            ),
        }
    }
}
