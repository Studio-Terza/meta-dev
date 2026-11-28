use std::iter;
use std::ops::ControlFlow;

use logos::{Lexer, Logos};

use super::lexeme::{Here, Inside, Lexeme, Line, Spelling};
use super::{Expanding, Part, Quoting, Simple, Unreadable, Word};
use crate::Whole;

enum Lexing<'text> {
    Bare(Lexer<'text, Lexeme<'text>>),
    Inside(Lexer<'text, Inside<'text>>),
}

pub(super) struct Reading<'text> {
    lexing: Lexing<'text>,
    words: Words,
    closer: Closer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Closer {
    End,
    Paren,
    Backtick,
}

#[derive(Debug)]
struct Enclosed {
    simples: Vec<Simple>,
    length: usize,
}

#[derive(Debug, Default)]
struct Words {
    read: Vec<Simple>,
    simple: Simple,
    word: Option<Word>,
    redirect: Option<Redirect>,
    pending: Vec<Pending>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Redirect {
    Input,
    Output,
    Heredoc(Strip),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strip {
    Tabs,
    Kept,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fed {
    Script,
    Data,
}

#[derive(Debug)]
struct Pending {
    delimiter: String,
    strip: Strip,
    quoting: Quoting,
    owner: usize,
    fed: Fed,
}

#[derive(Debug)]
struct Feeding {
    read: Vec<Simple>,
    length: usize,
}

#[derive(Debug)]
struct Bodied<'text> {
    text: &'text str,
    length: usize,
}

#[derive(Clone, Copy, Debug)]
struct Scan<'text> {
    start: usize,
    seen: Seen<'text>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Seen<'text> {
    Nothing,
    Tabs,
    Text(&'text str),
    Indented(&'text str),
}

struct Data<'text> {
    lexer: Lexer<'text, Here>,
    scripts: Vec<Vec<Simple>>,
}

impl<'text> Reading<'text> {
    pub(super) fn simples(text: &'text str) -> Result<Vec<Simple>, Unreadable> {
        Reading::enclosed(text, Closer::End).map(|enclosed| enclosed.simples)
    }

    fn enclosed(text: &'text str, closer: Closer) -> Result<Enclosed, Unreadable> {
        let first = Reading { lexing: Lexing::Bare(Lexeme::lexer(text)), words: Words::default(), closer };
        match iter::repeat(()).try_fold(first, |reading, ()| reading.step()) {
            ControlFlow::Break(read) => read,
            ControlFlow::Continue(_endless) => Ok(Enclosed { simples: Vec::new(), length: 0 }),
        }
    }

    fn step(self) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        let Reading { lexing, words, closer } = self;
        match lexing {
            Lexing::Bare(mut lexer) => match lexer.next() {
                Some(Ok(lexeme)) => words.bare(lexeme, lexer, closer),
                Some(Err(())) => Words::refused(lexer.slice()),
                None => match closer {
                    Closer::End => words.finished(lexer.source().len()),
                    Closer::Paren | Closer::Backtick => Words::refused(Words::UNCLOSED),
                },
            },
            Lexing::Inside(mut lexer) => match lexer.next() {
                Some(Ok(inside)) => words.inside(inside, lexer, closer),
                Some(Err(())) => Words::refused(lexer.slice()),
                None => Words::refused(Words::QUOTE),
            },
        }
    }
}

impl Words {
    const QUOTE: &'static str = "\"";
    const UNCLOSED: &'static str = "a `$(` or backtick that is never closed";
    const UNBODIED: &'static str = "a heredoc whose closing delimiter never comes";
    const UNAIMED: &'static str = "a redirection with no target";
    const GLOBS: [char; 3] = ['*', '?', '['];

    fn refused<'text>(span: &str) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        ControlFlow::Break(Err(Unreadable { span: span.to_owned() }))
    }

    fn bare<'text>(
        self,
        lexeme: Lexeme<'text>,
        mut lexer: Lexer<'text, Lexeme<'text>>,
        closer: Closer,
    ) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        let slice = lexer.slice();
        match lexeme {
            Lexeme::Blank | Lexeme::Duplicate => self.worded().barely(lexer, closer),
            Lexeme::Separator => match self.worded().separated() {
                Ok(separated) => separated.barely(lexer, closer),
                Err(unreadable) => ControlFlow::Break(Err(unreadable)),
            },
            Lexeme::Newline => match self.worded().separated() {
                Ok(separated) => separated.bodied(lexer, closer),
                Err(unreadable) => ControlFlow::Break(Err(unreadable)),
            },
            Lexeme::Input => self.worded().redirected(Redirect::Input, lexer, closer),
            Lexeme::Output => self.worded().redirected(Redirect::Output, lexer, closer),
            Lexeme::Heredoc => self.worded().redirected(Redirect::Heredoc(Strip::Kept), lexer, closer),
            Lexeme::Stripped => self.worded().redirected(Redirect::Heredoc(Strip::Tabs), lexer, closer),
            Lexeme::Quote => self.part("", Quoting::Quoted, Expanding::Literal, None).inwardly(lexer.morph(), closer),
            Lexeme::Quoted(text) => self
                .part(text, Quoting::Quoted, Expanding::Literal, Some(Part::Text(text.to_owned())))
                .barely(lexer, closer),
            Lexeme::Variable(name) => {
                let part = Some(Part::Variable(name.to_owned()));
                self.part(slice, Quoting::Plain, Expanding::Expanded, part).barely(lexer, closer)
            }
            Lexeme::Opaque(text) => {
                self.part(text, Quoting::Plain, Expanding::Expanded, Some(Part::Opaque)).barely(lexer, closer)
            }
            Lexeme::Literal(text) => {
                let globbed = text.contains(Words::GLOBS);
                let expanding = if globbed { Expanding::Expanded } else { Expanding::Literal };
                let part = if globbed { Part::Opaque } else { Part::Text(text.to_owned()) };
                self.part(text, Quoting::Plain, expanding, Some(part)).barely(lexer, closer)
            }
            Lexeme::Tilde(text) => {
                let part = if self.word.is_none() { Part::Opaque } else { Part::Text(text.to_owned()) };
                self.part(text, Quoting::Plain, Expanding::Literal, Some(part)).barely(lexer, closer)
            }
            Lexeme::Substitution => match self.substituted(&mut lexer, Closer::Paren) {
                Ok(substituted) => substituted.barely(lexer, closer),
                Err(unreadable) => ControlFlow::Break(Err(unreadable)),
            },
            Lexeme::Backtick => match closer {
                Closer::Backtick => self.finished(lexer.span().end),
                Closer::End | Closer::Paren => match self.substituted(&mut lexer, Closer::Backtick) {
                    Ok(substituted) => substituted.barely(lexer, closer),
                    Err(unreadable) => ControlFlow::Break(Err(unreadable)),
                },
            },
            Lexeme::Close => match closer {
                Closer::Paren => self.finished(lexer.span().end),
                Closer::End | Closer::Backtick => Words::refused(slice),
            },
            Lexeme::Unplain => Words::refused(slice),
        }
    }

    fn inside<'text>(
        self,
        inside: Inside<'text>,
        mut lexer: Lexer<'text, Inside<'text>>,
        closer: Closer,
    ) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        let slice = lexer.slice();
        match inside {
            Inside::End => self.barely(lexer.morph(), closer),
            Inside::Text(text) => {
                let part = Some(Part::Text(text.to_owned()));
                self.part(text, Quoting::Quoted, Expanding::Literal, part).inwardly(lexer, closer)
            }
            Inside::Variable(name) => {
                let part = Some(Part::Variable(name.to_owned()));
                self.part(slice, Quoting::Quoted, Expanding::Expanded, part).inwardly(lexer, closer)
            }
            Inside::Opaque(text) => {
                self.part(text, Quoting::Quoted, Expanding::Expanded, Some(Part::Opaque)).inwardly(lexer, closer)
            }
            Inside::Substitution => match self.substituted(&mut lexer, Closer::Paren) {
                Ok(substituted) => substituted.inwardly(lexer, closer),
                Err(unreadable) => ControlFlow::Break(Err(unreadable)),
            },
            Inside::Backtick => match self.substituted(&mut lexer, Closer::Backtick) {
                Ok(substituted) => substituted.inwardly(lexer, closer),
                Err(unreadable) => ControlFlow::Break(Err(unreadable)),
            },
            Inside::Unplain => Words::refused(slice),
        }
    }

    fn barely<'text>(
        self,
        lexer: Lexer<'text, Lexeme<'text>>,
        closer: Closer,
    ) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        ControlFlow::Continue(Reading { lexing: Lexing::Bare(lexer), words: self, closer })
    }

    fn inwardly<'text>(
        self,
        lexer: Lexer<'text, Inside<'text>>,
        closer: Closer,
    ) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        ControlFlow::Continue(Reading { lexing: Lexing::Inside(lexer), words: self, closer })
    }

    fn substituted<'text, Token: Logos<'text, Source = str>>(
        self,
        lexer: &mut Lexer<'text, Token>,
        inner: Closer,
    ) -> Result<Words, Unreadable> {
        let opened = lexer.span().start;
        let start = lexer.span().end;
        let source = lexer.source();
        let enclosed = Reading::enclosed(source.get(start..).unwrap_or_default(), inner)?;
        lexer.bump(enclosed.length);
        let text = source.get(opened..start + enclosed.length).unwrap_or_default();
        let parted = self.part(text, Quoting::Plain, Expanding::Expanded, Some(Part::Opaque));
        Ok(Words { simple: parted.simple.scripted(enclosed.simples), ..parted })
    }

    fn part(self, text: &str, quoting: Quoting, expanding: Expanding, part: Option<Part>) -> Words {
        let begun = self.word.unwrap_or_default();
        Words { word: Some(begun.extended(text, quoting, expanding, part)), ..self }
    }

    fn redirected<'text>(
        self,
        redirect: Redirect,
        lexer: Lexer<'text, Lexeme<'text>>,
        closer: Closer,
    ) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        match self.redirect {
            Some(_pending) => Words::refused(lexer.slice()),
            None => Words { redirect: Some(redirect), ..self }.barely(lexer, closer),
        }
    }

    fn worded(self) -> Words {
        let Words { read, simple, word, redirect, pending } = self;
        match (word, redirect) {
            (None, _) => Words { read, simple, word: None, redirect, pending },
            (Some(_input), Some(Redirect::Input)) => Words { read, simple, word: None, redirect: None, pending },
            (Some(target), Some(Redirect::Output)) => {
                Words { read, simple: simple.writing(target), word: None, redirect: None, pending }
            }
            (Some(delimiter), Some(Redirect::Heredoc(strip))) => {
                let awaited = Pending {
                    delimiter: delimiter.text,
                    strip,
                    quoting: delimiter.quoting,
                    owner: read.len(),
                    fed: simple.fed(),
                };
                Words {
                    read,
                    simple,
                    word: None,
                    redirect: None,
                    pending: pending.into_iter().chain([awaited]).collect(),
                }
            }
            (Some(argument), None) => {
                Words { read, simple: simple.worded(argument), word: None, redirect: None, pending }
            }
        }
    }

    fn separated(self) -> Result<Words, Unreadable> {
        let Words { read, simple, word, redirect, pending } = self;
        let unaimed = redirect.map(|_pending| Unreadable { span: Words::UNAIMED.to_owned() });
        let keyword = simple.keyword().map(|head| Unreadable { span: head.text.clone() });
        unaimed.or(keyword).map_or(Ok(()), Err)?;
        let kept = read.into_iter().chain([simple]).collect();
        Ok(Words { read: kept, simple: Simple::default(), word, redirect: None, pending })
    }

    fn finished<'text>(self, length: usize) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        match self.worded().separated() {
            Ok(words) if words.pending.is_empty() => ControlFlow::Break(Ok(Enclosed { simples: words.read, length })),
            Ok(_unbodied) => Words::refused(Words::UNBODIED),
            Err(unreadable) => ControlFlow::Break(Err(unreadable)),
        }
    }

    fn bodied<'text>(
        self,
        mut lexer: Lexer<'text, Lexeme<'text>>,
        closer: Closer,
    ) -> ControlFlow<Result<Enclosed, Unreadable>, Reading<'text>> {
        let rest = lexer.source().get(lexer.span().end..).unwrap_or_default();
        let Words { read, pending, .. } = self;
        let fed = pending.iter().try_fold(Feeding { read, length: 0 }, |feeding, awaited| feeding.fed(awaited, rest));
        match fed {
            Ok(feeding) => {
                lexer.bump(feeding.length);
                Words { read: feeding.read, ..Words::default() }.barely(lexer, closer)
            }
            Err(unreadable) => ControlFlow::Break(Err(unreadable)),
        }
    }
}

impl Feeding {
    fn fed(self, awaited: &Pending, rest: &str) -> Result<Feeding, Unreadable> {
        let unbodied = || Unreadable { span: Words::UNBODIED.to_owned() };
        let body = Bodied::read(rest.get(self.length..).unwrap_or_default(), awaited).ok_or_else(unbodied)?;
        let scripts = match (awaited.fed, awaited.quoting) {
            (Fed::Script, Quoting::Plain | Quoting::Quoted) => vec![Reading::simples(body.text)?],
            (Fed::Data, Quoting::Plain) => Data::scripts(body.text)?,
            (Fed::Data, Quoting::Quoted) => Vec::new(),
        };
        let read = self
            .read
            .into_iter()
            .enumerate()
            .map(|(at, simple)| if at == awaited.owner { simple.scripted_all(&scripts) } else { simple })
            .collect();
        Ok(Feeding { read, length: self.length + body.length })
    }
}

impl<'text> Bodied<'text> {
    fn read(text: &'text str, awaited: &Pending) -> Option<Bodied<'text>> {
        let first = Scan { start: 0, seen: Seen::Nothing };
        let walked = Line::lexer(text).spanned().try_fold(first, |scan, (line, span)| match line {
            Ok(Line::Tabs) => ControlFlow::Continue(Scan { seen: scan.seen.tabbed(), ..scan }),
            Ok(Line::Text(read)) => ControlFlow::Continue(Scan { seen: scan.seen.texted(read), ..scan }),
            Ok(Line::End) => match scan.ends(awaited) {
                Some(()) => {
                    ControlFlow::Break(Bodied { text: text.get(..scan.start).unwrap_or_default(), length: span.end })
                }
                None => ControlFlow::Continue(Scan { start: span.end, seen: Seen::Nothing }),
            },
            Err(()) => ControlFlow::Continue(scan),
        });
        match walked {
            ControlFlow::Break(bodied) => Some(bodied),
            ControlFlow::Continue(last) => (last.start < text.len())
                .then_some(())
                .and_then(|()| last.ends(awaited))
                .map(|()| Bodied { text: text.get(..last.start).unwrap_or_default(), length: text.len() }),
        }
    }
}

impl Scan<'_> {
    fn ends(&self, awaited: &Pending) -> Option<()> {
        let line = match (self.seen, awaited.strip) {
            (Seen::Nothing, Strip::Tabs | Strip::Kept) | (Seen::Tabs, Strip::Tabs) => Some(""),
            (Seen::Text(text), Strip::Tabs | Strip::Kept) | (Seen::Indented(text), Strip::Tabs) => Some(text),
            (Seen::Tabs | Seen::Indented(_), Strip::Kept) => None,
        };
        line.filter(|read| *read == awaited.delimiter).map(|_read| ())
    }
}

impl<'text> Seen<'text> {
    fn tabbed(self) -> Seen<'text> {
        match self {
            Seen::Nothing => Seen::Tabs,
            Seen::Tabs | Seen::Text(_) | Seen::Indented(_) => self,
        }
    }

    fn texted(self, text: &'text str) -> Seen<'text> {
        match self {
            Seen::Nothing => Seen::Text(text),
            Seen::Tabs => Seen::Indented(text),
            Seen::Text(_) | Seen::Indented(_) => self,
        }
    }
}

impl<'text> Data<'text> {
    fn scripts(body: &'text str) -> Result<Vec<Vec<Simple>>, Unreadable> {
        let first = Data { lexer: Here::lexer(body), scripts: Vec::new() };
        match iter::repeat(()).try_fold(first, |data, ()| data.step()) {
            ControlFlow::Break(read) => read,
            ControlFlow::Continue(_endless) => Ok(Vec::new()),
        }
    }

    fn step(self) -> ControlFlow<Result<Vec<Vec<Simple>>, Unreadable>, Data<'text>> {
        let Data { mut lexer, scripts } = self;
        let unreadable = |span: &str| ControlFlow::Break(Err(Unreadable { span: span.to_owned() }));
        match lexer.next() {
            None => ControlFlow::Break(Ok(scripts)),
            Some(Ok(Here::Text)) => ControlFlow::Continue(Data { lexer, scripts }),
            Some(Ok(Here::Substitution)) => Data::enclosing(lexer, scripts, Closer::Paren),
            Some(Ok(Here::Backtick)) => Data::enclosing(lexer, scripts, Closer::Backtick),
            Some(Ok(Here::Unplain) | Err(())) => unreadable(lexer.slice()),
        }
    }

    fn enclosing(
        mut lexer: Lexer<'text, Here>,
        scripts: Vec<Vec<Simple>>,
        closer: Closer,
    ) -> ControlFlow<Result<Vec<Vec<Simple>>, Unreadable>, Data<'text>> {
        let rest = lexer.source().get(lexer.span().end..).unwrap_or_default();
        match Reading::enclosed(rest, closer) {
            Ok(enclosed) => {
                lexer.bump(enclosed.length);
                ControlFlow::Continue(Data { lexer, scripts: scripts.into_iter().chain([enclosed.simples]).collect() })
            }
            Err(unreadable) => ControlFlow::Break(Err(unreadable)),
        }
    }
}

impl Simple {
    fn writing(self, target: Word) -> Simple {
        Simple { written: self.written.into_iter().chain([target]).collect(), ..self }
    }

    fn worded(self, argument: Word) -> Simple {
        Simple { words: self.words.into_iter().chain([argument]).collect(), ..self }
    }

    fn scripted(self, script: Vec<Simple>) -> Simple {
        Simple { scripts: self.scripts.into_iter().chain([script]).collect(), ..self }
    }

    fn scripted_all(self, scripts: &[Vec<Simple>]) -> Simple {
        Simple { scripts: self.scripts.into_iter().chain(scripts.iter().cloned()).collect(), ..self }
    }

    fn head(&self) -> Option<&Word> {
        self.words.iter().find(|word| Whole::exactly(&word.text) != Some([Spelling::Assignment]))
    }

    fn keyword(&self) -> Option<&Word> {
        self.head()
            .filter(|word| word.quoting == Quoting::Plain && Whole::exactly(&word.text) == Some([Spelling::Keyword]))
    }

    fn fed(&self) -> Fed {
        let shelled = self.head().and_then(|word| Whole::exactly(&word.text)) == Some([Spelling::Shell]);
        if shelled { Fed::Script } else { Fed::Data }
    }
}

impl Word {
    fn extended(self, text: &str, quoting: Quoting, expanding: Expanding, part: Option<Part>) -> Word {
        Word {
            text: [self.text.as_str(), text].concat(),
            quoting: self.quoting.max(quoting),
            expanding: self.expanding.max(expanding),
            parts: self.parts.into_iter().chain(part).collect(),
        }
    }
}
