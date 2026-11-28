mod lexeme;
mod reading;

use std::path::Path;

use lexeme::{Argued, PerlFlag, SedFlag, Spelling, Whole};
use reading::Reading;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plain {
    unruled: Vec<Deletion>,
    edits: Vec<Edited>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    span: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deletion {
    Rm,
    Rmdir,
    FindDelete,
    CargoClean,
    GitClean,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editor {
    Sed,
    Perl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    InPlace(Editor),
    Wrote(String),
    Moved { sources: Vec<String>, destination: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edited {
    edit: Edit,
    directory: Directory,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Directory {
    #[default]
    Started,
    Changed(Vec<String>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Changed {
    To(String),
    Unknown,
}

#[derive(Debug, Default)]
struct Simple {
    words: Vec<Word>,
    written: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct Word {
    text: String,
    quoting: Quoting,
    expanding: Expanding,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Quoting {
    #[default]
    Plain,
    Quoted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Expanding {
    #[default]
    Literal,
    Expanded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Depth {
    Top,
    Nested,
}

#[derive(Debug, Default)]
struct Walk {
    directory: Directory,
    unruled: Vec<Deletion>,
    edits: Vec<Edited>,
}

#[derive(Debug, Default)]
struct Found {
    unruled: Vec<Deletion>,
    edits: Vec<Edit>,
    scripted: Vec<Edited>,
    changed: Option<Changed>,
}

#[derive(Debug, Default)]
struct Exec<'words> {
    found: Found,
    run: Option<Vec<&'words Word>>,
}

impl Plain {
    pub fn read(command: &str) -> Result<Plain, Unreadable> {
        Plain::scanned(command, Depth::Top)
    }

    #[must_use]
    pub fn unruled(&self) -> &[Deletion] {
        &self.unruled
    }

    #[must_use]
    pub fn edits(&self) -> &[Edited] {
        &self.edits
    }

    fn scanned(command: &str, depth: Depth) -> Result<Plain, Unreadable> {
        let simples = Reading::simples(command)?;
        let walked = simples.iter().try_fold(Walk::default(), |walk, simple| walk.stepped(simple, depth))?;
        Ok(Plain { unruled: walked.unruled, edits: walked.edits })
    }
}

impl Unreadable {
    #[must_use]
    pub fn span(&self) -> &str {
        &self.span
    }
}

impl Edited {
    #[must_use]
    pub fn directory(&self) -> &Directory {
        &self.directory
    }

    #[must_use]
    pub fn edit(&self) -> &Edit {
        &self.edit
    }
}

impl Walk {
    fn stepped(self, simple: &Simple, depth: Depth) -> Result<Walk, Unreadable> {
        let words: Vec<&Word> = simple.words.iter().collect();
        let found = Found::command(&words, depth)?;
        let here = &self.directory;
        let wrote = simple.written.iter().map(|written| Edit::Wrote(written.clone()));
        let local = wrote.chain(found.edits).map(|edit| Edited { edit, directory: here.clone() });
        let scripted = found.scripted.into_iter().map(|edited| edited.beneath(here));
        let next = found.changed.map_or_else(|| here.clone(), |changed| here.clone().changed(changed));
        Ok(Walk {
            directory: next,
            unruled: self.unruled.into_iter().chain(found.unruled).collect(),
            edits: self.edits.into_iter().chain(local).chain(scripted).collect(),
        })
    }
}

impl Found {
    const OPAQUE: [char; 3] = ['$', '`', '~'];

    fn command(words: &[&Word], depth: Depth) -> Result<Found, Unreadable> {
        let spelled = |word: &&Word| Whole::read::<Spelling>(&word.text);
        let run: Vec<&Word> =
            words.iter().copied().skip_while(|word| spelled(word) == Some(Spelling::Assignment)).collect();
        match run.split_first() {
            Some((head, arguments)) => match spelled(head) {
                Some(Spelling::Rm) => Ok(Found::deleted(Deletion::Rm, arguments, depth)),
                Some(Spelling::Rmdir) => Ok(Found::deleted(Deletion::Rmdir, arguments, depth)),
                Some(Spelling::Cargo) => Ok(Found::cleaned(Deletion::CargoClean, arguments, depth)),
                Some(Spelling::Git) => Ok(Found::cleaned(Deletion::GitClean, arguments, depth)),
                Some(Spelling::Find) => Found::searched(arguments, depth),
                Some(Spelling::Xargs) => Found::command(Found::utility(arguments), Depth::Nested),
                Some(Spelling::Shell) => Found::shelled(arguments),
                Some(Spelling::Sed) => Ok(Found::edited(arguments, Editor::Sed)),
                Some(Spelling::Perl) => Ok(Found::edited(arguments, Editor::Perl)),
                Some(Spelling::Tee) => {
                    let edits = Found::paths(arguments).map(|path| Edit::Wrote(path.text.clone())).collect();
                    Ok(Found { edits, ..Found::default() })
                }
                Some(Spelling::Mv) => Ok(Found::moved(arguments)),
                Some(Spelling::Cd) => Ok(Found::cd(arguments)),
                Some(Spelling::Keyword | Spelling::Assignment) | None => Ok(Found::default()),
            },
            None => Ok(Found::default()),
        }
    }

    fn joined(self, other: Found) -> Found {
        Found {
            unruled: self.unruled.into_iter().chain(other.unruled).collect(),
            edits: self.edits.into_iter().chain(other.edits).collect(),
            scripted: self.scripted.into_iter().chain(other.scripted).collect(),
            changed: other.changed.or(self.changed),
        }
    }

    fn deleted(deletion: Deletion, arguments: &[&Word], depth: Depth) -> Found {
        let expanded = arguments.iter().any(|word| word.expanding == Expanding::Expanded);
        let unruled = (depth == Depth::Nested || expanded).then_some(deletion);
        Found { unruled: unruled.into_iter().collect(), ..Found::default() }
    }

    fn cleaned(deletion: Deletion, arguments: &[&Word], depth: Depth) -> Found {
        let sub = arguments.iter().find(|word| {
            !matches!(Whole::read::<Argued>(&word.text), Some(Argued::Flag | Argued::Script | Argued::Valued))
        });
        match sub.and_then(|word| Whole::read::<Argued>(&word.text)) {
            Some(Argued::Clean) => Found::deleted(deletion, arguments, depth),
            Some(
                Argued::Exec | Argued::Delete | Argued::Terminator | Argued::Valued | Argued::Script | Argued::Flag,
            )
            | None => Found::default(),
        }
    }

    fn searched(arguments: &[&Word], depth: Depth) -> Result<Found, Unreadable> {
        let deleting = arguments.iter().any(|word| Whole::read::<Argued>(&word.text) == Some(Argued::Delete));
        let deleted = if deleting { Found::deleted(Deletion::FindDelete, arguments, depth) } else { Found::default() };
        let exec = arguments.iter().try_fold(Exec { found: deleted, run: None }, |exec, word| exec.read(word))?;
        exec.ended()
    }

    fn utility<'words, 'word>(arguments: &'words [&'word Word]) -> &'words [&'word Word] {
        match arguments.split_first() {
            Some((first, rest)) => match Whole::read::<Argued>(&first.text) {
                Some(Argued::Valued) => Found::utility(rest.split_first().map_or(rest, |(_value, after)| after)),
                Some(Argued::Flag | Argued::Script | Argued::Exec | Argued::Delete) => Found::utility(rest),
                Some(Argued::Terminator | Argued::Clean) | None => arguments,
            },
            None => arguments,
        }
    }

    fn shelled(arguments: &[&Word]) -> Result<Found, Unreadable> {
        match arguments.split_first() {
            Some((first, rest)) => match Whole::read::<Argued>(&first.text) {
                Some(Argued::Script) => {
                    rest.first().map_or(Ok(Found::default()), |script| Found::scripted(&script.text))
                }
                Some(Argued::Flag | Argued::Valued | Argued::Exec | Argued::Delete) => Found::shelled(rest),
                Some(Argued::Terminator | Argued::Clean) | None => Ok(Found::default()),
            },
            None => Ok(Found::default()),
        }
    }

    fn scripted(script: &str) -> Result<Found, Unreadable> {
        let plain = Plain::scanned(script, Depth::Nested)?;
        Ok(Found { unruled: plain.unruled, scripted: plain.edits, ..Found::default() })
    }

    fn edited(arguments: &[&Word], editor: Editor) -> Found {
        let in_place = arguments.iter().any(|word| match editor {
            Editor::Sed => Whole::read::<SedFlag>(&word.text) == Some(SedFlag::InPlace),
            Editor::Perl => Whole::read::<PerlFlag>(&word.text) == Some(PerlFlag::InPlace),
        });
        let edits = in_place.then_some(Edit::InPlace(editor)).into_iter().collect();
        Found { edits, ..Found::default() }
    }

    fn paths<'words, 'word>(
        arguments: &'words [&'word Word],
    ) -> impl Iterator<Item = &'word Word> + use<'words, 'word> {
        arguments.iter().copied().filter(|word| Whole::read::<PerlFlag>(&word.text).is_none())
    }

    fn moved(arguments: &[&Word]) -> Found {
        let paths: Vec<String> = Found::paths(arguments).map(|word| word.text.clone()).collect();
        let moved = paths
            .split_last()
            .filter(|(_destination, sources)| !sources.is_empty())
            .map(|(destination, sources)| Edit::Moved { sources: sources.to_vec(), destination: destination.clone() });
        Found { edits: moved.into_iter().collect(), ..Found::default() }
    }

    fn cd(arguments: &[&Word]) -> Found {
        let target = Found::paths(arguments).next();
        let changed = target.map_or(Changed::Unknown, |word| {
            let opaque = word.expanding == Expanding::Expanded || word.text.contains(Found::OPAQUE);
            if opaque { Changed::Unknown } else { Changed::To(word.text.clone()) }
        });
        Found { changed: Some(changed), ..Found::default() }
    }
}

impl<'words> Exec<'words> {
    fn read(self, word: &'words Word) -> Result<Exec<'words>, Unreadable> {
        let argued = Whole::read::<Argued>(&word.text);
        match (self.run, argued) {
            (None, Some(Argued::Exec)) => Ok(Exec { found: self.found, run: Some(Vec::new()) }),
            (None, _) => Ok(Exec { found: self.found, run: None }),
            (Some(run), Some(Argued::Terminator)) => {
                let found = self.found.joined(Found::command(&run, Depth::Nested)?);
                Ok(Exec { found, run: None })
            }
            (Some(run), _) => Ok(Exec { found: self.found, run: Some(run.into_iter().chain([word]).collect()) }),
        }
    }

    fn ended(self) -> Result<Found, Unreadable> {
        match self.run {
            Some(run) => Ok(self.found.joined(Found::command(&run, Depth::Nested)?)),
            None => Ok(self.found),
        }
    }
}

impl Edited {
    fn beneath(self, outer: &Directory) -> Edited {
        let directory = match self.directory {
            Directory::Started => outer.clone(),
            Directory::Changed(steps) => {
                steps.into_iter().fold(outer.clone(), |directory, step| directory.changed(Changed::To(step)))
            }
            Directory::Unknown => Directory::Unknown,
        };
        Edited { edit: self.edit, directory }
    }
}

impl Directory {
    fn changed(self, changed: Changed) -> Directory {
        match (self, changed) {
            (_, Changed::To(step)) if Path::new(&step).is_absolute() => Directory::Changed(vec![step]),
            (Directory::Started, Changed::To(step)) => Directory::Changed(vec![step]),
            (Directory::Changed(steps), Changed::To(step)) => {
                Directory::Changed(steps.into_iter().chain([step]).collect())
            }
            (Directory::Unknown, Changed::To(_)) | (_, Changed::Unknown) => Directory::Unknown,
        }
    }
}
