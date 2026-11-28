mod lexeme;
mod reading;

use std::fmt;
use std::path::Path;

use lexeme::{Argued, Assigning, PerlFlag, Prefix, SedFlag, Spelling, Whole};
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
    Wrote(Target),
    Moved { sources: Vec<Target>, destination: Target },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    text: String,
    parts: Vec<Part>,
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
    Changed(Vec<Target>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Part {
    Text(String),
    Variable(String),
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Changed {
    To(Target),
    Unknown,
}

#[derive(Clone, Debug, Default)]
struct Simple {
    words: Vec<Word>,
    written: Vec<Word>,
    scripts: Vec<Vec<Simple>>,
}

#[derive(Clone, Debug, Default)]
struct Word {
    text: String,
    quoting: Quoting,
    expanding: Expanding,
    parts: Vec<Part>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Assigned {
    name: String,
    parts: Vec<Part>,
}

#[derive(Clone, Copy, Debug)]
struct Scope<'walk> {
    depth: Depth,
    assigned: &'walk [Assigned],
}

#[derive(Debug)]
struct Prefixed<'text, Token> {
    token: Token,
    rest: &'text str,
}

#[derive(Debug, Default)]
struct Walk {
    directory: Directory,
    assigned: Vec<Assigned>,
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
        let scope = Scope { depth: Depth::Top, assigned: &[] };
        Plain::scanned(command, scope)
    }

    #[must_use]
    pub fn unruled(&self) -> &[Deletion] {
        &self.unruled
    }

    #[must_use]
    pub fn edits(&self) -> &[Edited] {
        &self.edits
    }

    fn scanned(command: &str, scope: Scope<'_>) -> Result<Plain, Unreadable> {
        Plain::walked(&Reading::simples(command)?, scope)
    }

    fn walked(simples: &[Simple], scope: Scope<'_>) -> Result<Plain, Unreadable> {
        let first = Walk { assigned: scope.assigned.to_vec(), ..Walk::default() };
        let walked = simples.iter().try_fold(first, |walk, simple| walk.stepped(simple, scope.depth))?;
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

impl Target {
    #[must_use]
    pub fn resolved(&self, variable: impl Fn(&str) -> Option<String>) -> Option<String> {
        self.parts
            .iter()
            .map(|part| match part {
                Part::Text(text) => Some(text.clone()),
                Part::Variable(name) => variable(name),
                Part::Opaque => None,
            })
            .collect()
    }

    fn of(word: &Word, assigned: &[Assigned]) -> Target {
        let parts = word.parts.iter().flat_map(|part| Part::substituted(part, assigned)).collect();
        Target { text: word.text.clone(), parts }
    }

    fn opaque(&self) -> Option<()> {
        self.parts.contains(&Part::Opaque).then_some(())
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl Part {
    fn substituted(part: &Part, assigned: &[Assigned]) -> Vec<Part> {
        match part {
            Part::Variable(name) => assigned
                .iter()
                .rev()
                .find(|earlier| earlier.name == *name)
                .map_or_else(|| vec![part.clone()], |earlier| earlier.parts.clone()),
            Part::Text(_) | Part::Opaque => vec![part.clone()],
        }
    }
}

impl Walk {
    fn stepped(self, simple: &Simple, depth: Depth) -> Result<Walk, Unreadable> {
        let words: Vec<&Word> = simple.words.iter().collect();
        let scope = Scope { depth, assigned: &self.assigned };
        let found = Found::command(&words, scope)?;
        let nested = Scope { depth: Depth::Nested, assigned: &self.assigned };
        let scripts =
            simple.scripts.iter().map(|script| Plain::walked(script, nested)).collect::<Result<Vec<_>, _>>()?;
        let here = &self.directory;
        let wrote = simple.written.iter().map(|written| Edit::Wrote(Target::of(written, &self.assigned)));
        let local = wrote.chain(found.edits).map(|edit| Edited { edit, directory: here.clone() });
        let inner_edits = scripts.iter().flat_map(|plain| plain.edits.iter().cloned());
        let scripted = found.scripted.into_iter().chain(inner_edits).map(|edited| edited.beneath(here));
        let inner_unruled = scripts.iter().flat_map(|plain| plain.unruled.iter().copied());
        let next = found.changed.map_or_else(|| here.clone(), |changed| here.clone().changed(changed));
        let assigned = Assigned::recorded(&words, &self.assigned);
        Ok(Walk {
            directory: next,
            unruled: self.unruled.into_iter().chain(found.unruled).chain(inner_unruled).collect(),
            edits: self.edits.into_iter().chain(local).chain(scripted).collect(),
            assigned: self.assigned.iter().cloned().chain(assigned).collect(),
        })
    }
}

impl Assigned {
    fn recorded(words: &[&Word], earlier: &[Assigned]) -> Vec<Assigned> {
        let spelled = |word: &&&Word| Whole::read::<Spelling>(&word.text);
        let exported = words.first().and_then(|first| spelled(&first)) == Some(Spelling::Export);
        let assignments: Vec<&Word> = words.iter().copied().skip(usize::from(exported)).collect();
        let only = assignments.iter().all(|word| spelled(&word) == Some(Spelling::Assignment));
        let read = assignments.iter().filter_map(|word| Assigned::read(word, earlier));
        if only { read.collect() } else { Vec::new() }
    }

    fn read(word: &Word, earlier: &[Assigned]) -> Option<Assigned> {
        let (first, rest) = word.parts.split_first()?;
        let text = match first {
            Part::Text(text) => Some(text),
            Part::Variable(_) | Part::Opaque => None,
        }?;
        let prefixed: Prefixed<'_, Assigning<'_>> = Prefix::read(text)?;
        let Assigning::Name(name) = prefixed.token;
        let head = Some(prefixed.rest).filter(|value| !value.is_empty()).map(|value| Part::Text(value.to_owned()));
        let value = head.into_iter().chain(rest.iter().cloned());
        let parts = value.flat_map(|part| Part::substituted(&part, earlier)).collect();
        Some(Assigned { name: name.to_owned(), parts })
    }
}

impl Found {
    fn command(words: &[&Word], scope: Scope<'_>) -> Result<Found, Unreadable> {
        let spelled = |word: &&Word| Whole::read::<Spelling>(&word.text);
        let run: Vec<&Word> =
            words.iter().copied().skip_while(|word| spelled(word) == Some(Spelling::Assignment)).collect();
        let depth = scope.depth;
        match run.split_first() {
            Some((head, arguments)) => match spelled(head) {
                Some(Spelling::Rm) => Ok(Found::deleted(Deletion::Rm, arguments, depth)),
                Some(Spelling::Rmdir) => Ok(Found::deleted(Deletion::Rmdir, arguments, depth)),
                Some(Spelling::Cargo) => Ok(Found::cleaned(Deletion::CargoClean, arguments, depth)),
                Some(Spelling::Git) => Ok(Found::cleaned(Deletion::GitClean, arguments, depth)),
                Some(Spelling::Find) => Found::searched(arguments, scope),
                Some(Spelling::Xargs) => {
                    Found::command(Found::utility(arguments), Scope { depth: Depth::Nested, ..scope })
                }
                Some(Spelling::Shell) => Found::shelled(arguments, scope),
                Some(Spelling::Sed) => Ok(Found::edited(arguments, Editor::Sed)),
                Some(Spelling::Perl) => Ok(Found::edited(arguments, Editor::Perl)),
                Some(Spelling::Tee) => {
                    let edits =
                        Found::paths(arguments).map(|path| Edit::Wrote(Target::of(path, scope.assigned))).collect();
                    Ok(Found { edits, ..Found::default() })
                }
                Some(Spelling::Mv) => Ok(Found::moved(arguments, scope.assigned)),
                Some(Spelling::Cd) => Ok(Found::cd(arguments, scope.assigned)),
                Some(Spelling::Keyword | Spelling::Assignment | Spelling::Export) | None => Ok(Found::default()),
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

    fn searched(arguments: &[&Word], scope: Scope<'_>) -> Result<Found, Unreadable> {
        let deleting = arguments.iter().any(|word| Whole::read::<Argued>(&word.text) == Some(Argued::Delete));
        let deleted =
            if deleting { Found::deleted(Deletion::FindDelete, arguments, scope.depth) } else { Found::default() };
        let first = Exec { found: deleted, run: None };
        let exec = arguments.iter().try_fold(first, |exec, word| exec.read(word, scope))?;
        exec.ended(scope)
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

    fn shelled(arguments: &[&Word], scope: Scope<'_>) -> Result<Found, Unreadable> {
        match arguments.split_first() {
            Some((first, rest)) => match Whole::read::<Argued>(&first.text) {
                Some(Argued::Script) => {
                    rest.first().map_or(Ok(Found::default()), |script| Found::scripted(&script.text, scope))
                }
                Some(Argued::Flag | Argued::Valued | Argued::Exec | Argued::Delete) => Found::shelled(rest, scope),
                Some(Argued::Terminator | Argued::Clean) | None => Ok(Found::default()),
            },
            None => Ok(Found::default()),
        }
    }

    fn scripted(script: &str, scope: Scope<'_>) -> Result<Found, Unreadable> {
        let plain = Plain::scanned(script, Scope { depth: Depth::Nested, ..scope })?;
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

    fn moved(arguments: &[&Word], assigned: &[Assigned]) -> Found {
        let paths: Vec<Target> = Found::paths(arguments).map(|word| Target::of(word, assigned)).collect();
        let moved = paths
            .split_last()
            .filter(|(_destination, sources)| !sources.is_empty())
            .map(|(destination, sources)| Edit::Moved { sources: sources.to_vec(), destination: destination.clone() });
        Found { edits: moved.into_iter().collect(), ..Found::default() }
    }

    fn cd(arguments: &[&Word], assigned: &[Assigned]) -> Found {
        let target = Found::paths(arguments).next().map(|word| Target::of(word, assigned));
        let changed = target.map_or(Changed::Unknown, |step| match step.opaque() {
            Some(()) => Changed::Unknown,
            None => Changed::To(step),
        });
        Found { changed: Some(changed), ..Found::default() }
    }
}

impl<'words> Exec<'words> {
    fn read(self, word: &'words Word, scope: Scope<'_>) -> Result<Exec<'words>, Unreadable> {
        let argued = Whole::read::<Argued>(&word.text);
        let nested = Scope { depth: Depth::Nested, ..scope };
        match (self.run, argued) {
            (None, Some(Argued::Exec)) => Ok(Exec { found: self.found, run: Some(Vec::new()) }),
            (None, _) => Ok(Exec { found: self.found, run: None }),
            (Some(run), Some(Argued::Terminator)) => {
                let found = self.found.joined(Found::command(&run, nested)?);
                Ok(Exec { found, run: None })
            }
            (Some(run), _) => Ok(Exec { found: self.found, run: Some(run.into_iter().chain([word]).collect()) }),
        }
    }

    fn ended(self, scope: Scope<'_>) -> Result<Found, Unreadable> {
        let nested = Scope { depth: Depth::Nested, ..scope };
        match self.run {
            Some(run) => Ok(self.found.joined(Found::command(&run, nested)?)),
            None => Ok(self.found),
        }
    }
}

impl Directory {
    fn changed(self, changed: Changed) -> Directory {
        match (self, changed) {
            (_, Changed::To(step)) if Path::new(&step.text).is_absolute() => Directory::Changed(vec![step]),
            (Directory::Started, Changed::To(step)) => Directory::Changed(vec![step]),
            (Directory::Changed(steps), Changed::To(step)) => {
                Directory::Changed(steps.into_iter().chain([step]).collect())
            }
            (Directory::Unknown, Changed::To(_)) | (_, Changed::Unknown) => Directory::Unknown,
        }
    }
}
