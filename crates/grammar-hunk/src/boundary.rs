use std::path::Path;
use std::str::FromStr;

use grammar::logos::Logos;

use grammar_unspelled::Unspelled;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Line {
    #[regex(r"diff --git [^\n]*\n", allow_greedy = true)]
    Header,
    #[regex(
        r"(index|new file|deleted file|old mode|new mode|similarity|dissimilarity|rename|copy|Binary)[^\n]*\n",
        allow_greedy = true
    )]
    Meta,
    #[token("--- /dev/null\n", priority = 9)]
    OldAbsent,
    #[regex(r"--- a/[^\n]*\n", |lexer| Line::path(lexer.slice()), priority = 8, allow_greedy = true)]
    Old(String),
    #[token("+++ /dev/null\n", priority = 9)]
    NewAbsent,
    #[regex(r"\+\+\+ b/[^\n]*\n", priority = 8, allow_greedy = true)]
    New,
    #[regex(r"@@ [^\n]*\n", |lexer| Hunk::header(lexer.slice()), allow_greedy = true)]
    Range(Hunk),
    #[regex(r"-[^\n]*\n", priority = 2, allow_greedy = true)]
    Minus,
    #[regex(r"\+[^\n]*\n", priority = 2, allow_greedy = true)]
    Plus,
    #[regex(r" [^\n]*\n", allow_greedy = true)]
    Context,
    #[regex(r"\\[^\n]*\n", allow_greedy = true)]
    NoNewline,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Mark {
    #[regex("[0-9]+", |lexer| lexer.slice().parse().ok())]
    Number(u32),
    #[token(",")]
    Comma,
    #[token("@@ -")]
    Old,
    #[token(" +")]
    New,
    #[token(" @@")]
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Span {
    start: u32,
    len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hunk {
    old: Span,
    new: Span,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changed {
    old: Option<String>,
    hunks: Vec<Hunk>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Patch(Vec<Changed>);

#[derive(Debug, Default)]
struct Reading {
    changed: Vec<Changed>,
    left: u32,
}

impl Line {
    const PREFIX: usize = 6;

    fn path(slice: &str) -> Option<String> {
        slice.get(Self::PREFIX..slice.len().saturating_sub(1)).map(str::to_owned)
    }
}

impl Mark {
    fn number(self) -> Option<u32> {
        match self {
            Mark::Number(number) => Some(number),
            Mark::Comma | Mark::Old | Mark::New | Mark::Close => None,
        }
    }
}

impl Span {
    const ONE: u32 = 1;
    const COUNTED: usize = 2;

    fn read(marks: &[Mark]) -> Option<Span> {
        let (first, rest) = marks.split_first()?;
        let start = first.number()?;
        let len = rest.first().map_or(Some(Self::ONE), |_| Span::counted(rest))?;
        Some(Span { start, len })
    }

    fn counted(marks: &[Mark]) -> Option<u32> {
        let [comma, number] = <[Mark; Self::COUNTED]>::try_from(marks).ok()?;
        Some(comma).filter(|mark| *mark == Mark::Comma)?;
        number.number()
    }

    fn holds(self, line: u32) -> Option<()> {
        (self.start..self.start.saturating_add(self.len)).contains(&line).then_some(())
    }
}

impl Hunk {
    fn header(slice: &str) -> Option<Hunk> {
        let marks: Vec<Mark> =
            Mark::lexer(slice).map_while(Result::ok).take_while(|mark| *mark != Mark::Close).collect();
        let (opening, rest) = marks.split_first()?;
        Some(opening).filter(|mark| **mark == Mark::Old)?;
        let middle = rest.iter().position(|mark| *mark == Mark::New)?;
        let (old, new) = rest.split_at_checked(middle)?;
        Some(Hunk { old: Span::read(old)?, new: Span::read(new.get(1..)?)? })
    }

    fn before(&self, line: u32) -> Option<i64> {
        Some(self.old.start.saturating_add(self.old.len.max(Span::ONE)))
            .filter(|after| *after <= line)
            .map(|_| i64::from(self.new.len) - i64::from(self.old.len))
    }

    fn lines(&self) -> u32 {
        self.old.len.saturating_add(self.new.len)
    }

    fn removes(&self) -> Option<()> {
        Some(self.old.len).filter(|len| *len > 0).map(|_| ())
    }
}

impl Changed {
    #[must_use]
    pub fn before(&self) -> Option<&str> {
        self.old.as_deref()
    }

    #[must_use]
    pub fn removing(&self) -> Option<&Path> {
        self.hunks.iter().find_map(Hunk::removes)?;
        self.old.as_deref().map(Path::new)
    }

    #[must_use]
    pub fn moved(&self, line: u32) -> Option<u32> {
        self.hunks.iter().find_map(|hunk| hunk.old.holds(line)).map_or(Some(()), |()| None)?;
        let shift: i64 = self.hunks.iter().filter_map(|hunk| hunk.before(line)).sum();
        u32::try_from(i64::from(line) + shift).ok()
    }
}

impl Patch {
    #[must_use]
    pub fn changed(&self) -> &[Changed] {
        &self.0
    }
}

impl Reading {
    fn read(self, line: Line) -> Option<Reading> {
        match line {
            Line::Header => {
                Some(Reading { changed: self.changed.into_iter().chain([Changed::default()]).collect(), ..self })
            }
            Line::Meta | Line::NoNewline => Some(self),
            Line::OldAbsent | Line::NewAbsent | Line::New => self.content_or(|changed| changed),
            Line::Old(path) => self.content_or(|changed| Changed { old: Some(path), ..changed }),
            Line::Range(hunk) => self.last(
                |changed| Changed { hunks: changed.hunks.into_iter().chain([hunk]).collect(), ..changed },
                hunk.lines(),
            ),
            Line::Minus | Line::Plus | Line::Context => self.counted().ok(),
        }
    }

    fn counted(self) -> Result<Reading, Reading> {
        match Some(self.left).filter(|left| *left > 0) {
            Some(left) => Ok(Reading { left: left - 1, ..self }),
            None => Err(self),
        }
    }

    fn content_or(self, change: impl FnOnce(Changed) -> Changed) -> Option<Reading> {
        self.counted().or_else(|reading| reading.last(change, 0).ok_or(())).ok()
    }

    fn last(self, change: impl FnOnce(Changed) -> Changed, left: u32) -> Option<Reading> {
        let mut changed = self.changed;
        let opened = changed.pop()?;
        changed.push(change(opened));
        Some(Reading { changed, left })
    }
}

impl FromStr for Patch {
    type Err = Unspelled;

    fn from_str(text: &str) -> Result<Patch, Unspelled> {
        let lines: Result<Vec<Line>, ()> = Line::lexer(text).collect();
        lines
            .ok()
            .and_then(|lexed| lexed.into_iter().try_fold(Reading::default(), Reading::read))
            .filter(|read| read.left == 0)
            .map(|read| Patch(read.changed))
            .ok_or(Unspelled)
    }
}
