use std::ffi::OsStr;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;
use file_ports::{Files, Kind};
use grammar::shell::{Deletion, Directory, Edit, Edited, Editor, Plain, Target, Unreadable};
use serde::Deserialize;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Tape;
use super::brief::Brief;
use super::reply::{Hook, Reply};
use crate::cli::failure::Failure;
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
struct Judging<'ground, 'run, F: Files, S: Store, E: Environment> {
    ground: &'ground Ground<'run, F, S>,
    environment: &'ground E,
    cwd: &'ground Path,
}

impl Hook for Running {
    fn answered<F: Files + Clone, S: Store>(
        &self,
        environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'_>, Failure> {
        match &self.tool {
            Tool::Bash(input) => Ok(Judging { ground, environment, cwd: &self.cwd }.judged(input)),
            Tool::Agent(brief) | Tool::SendMessage(brief) => brief.told(ground, tape),
        }
    }
}

impl<F: Files, S: Store, E: Environment> Judging<'_, '_, F, S, E> {
    const REPOSITORY: &'static str = ".git";

    fn judged(&self, input: &Input) -> Reply<'static> {
        let command = input.command.as_deref().unwrap_or_default();
        let refused = match Plain::read(command) {
            Ok(plain) => plain
                .unruled()
                .first()
                .map(|deletion| Refusal::Unruled(*deletion))
                .or_else(|| plain.edits().iter().find_map(|edited| self.refusal(edited))),
            Err(unreadable) => Some(Refusal::Unreadable(unreadable)),
        };
        refused.map_or(Reply::Pass(Outcome::Hooked(HookEvent::PreToolUse)), |refusal| Reply::Deny {
            reason: refusal.to_string(),
        })
    }

    fn refusal(&self, edited: &Edited) -> Option<Refusal> {
        let directory = edited.directory();
        let unresolved = |written: &Target| Refusal::Unresolved { written: written.to_string() };
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
                    .map(unresolved);
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

    fn resolved(&self, written: &Target, directory: &Directory) -> Option<PathBuf> {
        let path = self.expanded(written)?;
        let absolute = Path::new(&path).is_absolute().then(PathBuf::new);
        let base = absolute.or_else(|| match directory {
            Directory::Started => Some(self.cwd.to_path_buf()),
            Directory::Changed(steps) => {
                steps.iter().try_fold(self.cwd.to_path_buf(), |base, step| Some(base.join(self.expanded(step)?)))
            }
            Directory::Unknown => None,
        })?;
        Some(Self::folded(&base.join(path)))
    }

    fn expanded(&self, target: &Target) -> Option<String> {
        target.resolved(|name| {
            self.environment.variable(OsStr::new(name)).and_then(|value| value.to_str().map(str::to_owned))
        })
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
                 $NAME, pipes and ; && ||, with `$(...)`, backticks and heredocs holding only such commands; no \
                 loops, conditionals, subshells, functions, arrays or arithmetic. Unroll the work into separate \
                 plain calls.",
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
                "Refused: a shell write or move onto {written}, whose path cannot be worked out (a variable that is \
                 not set, a glob or command output, or a `cd` whose target is unknown). Use absolute paths, or edit \
                 files with the Edit or Write tools."
            ),
        }
    }
}
