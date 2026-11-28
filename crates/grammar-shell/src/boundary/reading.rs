use std::iter;
use std::ops::ControlFlow;

use grammar::logos::{Lexer, Logos};

use super::lexeme::{Inside, Lexeme, Spelling, Whole};
use super::{Expanding, Quoting, Simple, Unreadable, Word};

enum Lexing<'text> {
    Bare(Lexer<'text, Lexeme<'text>>),
    Inside(Lexer<'text, Inside<'text>>),
}

pub(super) struct Reading<'text> {
    lexing: Lexing<'text>,
    words: Words,
}

type Step<'text> = ControlFlow<Result<Vec<Simple>, Unreadable>, Reading<'text>>;

#[derive(Debug, Default)]
struct Words {
    read: Vec<Simple>,
    simple: Simple,
    word: Option<Word>,
    redirect: Option<Redirect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Redirect {
    Input,
    Output,
}

impl<'text> Reading<'text> {
    pub(super) fn simples(text: &'text str) -> Result<Vec<Simple>, Unreadable> {
        let first = Reading { lexing: Lexing::Bare(Lexeme::lexer(text)), words: Words::default() };
        match iter::repeat(()).try_fold(first, |reading, ()| reading.step()) {
            ControlFlow::Break(read) => read,
            ControlFlow::Continue(_endless) => Ok(Vec::new()),
        }
    }

    fn step(self) -> Step<'text> {
        let Reading { lexing, words } = self;
        match lexing {
            Lexing::Bare(mut lexer) => match lexer.next() {
                Some(Ok(lexeme)) => words.bare(lexeme, lexer),
                Some(Err(())) => Words::refused(lexer.slice()),
                None => ControlFlow::Break(words.ended()),
            },
            Lexing::Inside(mut lexer) => match lexer.next() {
                Some(Ok(inside)) => words.inside(inside, lexer),
                Some(Err(())) => Words::refused(lexer.slice()),
                None => Words::refused(Words::QUOTE),
            },
        }
    }
}

impl Words {
    const QUOTE: &'static str = "\"";
    const GLOBS: [char; 3] = ['*', '?', '['];

    fn refused<'text>(span: &str) -> Step<'text> {
        ControlFlow::Break(Err(Unreadable { span: span.to_owned() }))
    }

    fn bare<'text>(self, lexeme: Lexeme<'text>, lexer: Lexer<'text, Lexeme<'text>>) -> Step<'text> {
        match lexeme {
            Lexeme::Blank | Lexeme::Duplicate => self.worded().barely(lexer),
            Lexeme::Separator => match self.worded().separated() {
                Ok(separated) => separated.barely(lexer),
                Err(unreadable) => ControlFlow::Break(Err(unreadable)),
            },
            Lexeme::Input => self.worded().redirected(Redirect::Input, lexer),
            Lexeme::Output => self.worded().redirected(Redirect::Output, lexer),
            Lexeme::Quote => self.part("", Quoting::Quoted, Expanding::Literal).inwardly(lexer.morph()),
            Lexeme::Quoted(text) => self.part(text, Quoting::Quoted, Expanding::Literal).barely(lexer),
            Lexeme::Expansion(text) => self.part(text, Quoting::Plain, Expanding::Expanded).barely(lexer),
            Lexeme::Literal(text) => {
                let expanding = if text.contains(Words::GLOBS) { Expanding::Expanded } else { Expanding::Literal };
                self.part(text, Quoting::Plain, expanding).barely(lexer)
            }
            Lexeme::Unplain => Words::refused(lexer.slice()),
        }
    }

    fn inside<'text>(self, inside: Inside<'text>, lexer: Lexer<'text, Inside<'text>>) -> Step<'text> {
        match inside {
            Inside::End => self.barely(lexer.morph()),
            Inside::Text(text) => self.part(text, Quoting::Quoted, Expanding::Literal).inwardly(lexer),
            Inside::Expansion(text) => self.part(text, Quoting::Quoted, Expanding::Expanded).inwardly(lexer),
            Inside::Unplain => Words::refused(lexer.slice()),
        }
    }

    fn barely<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Step<'text> {
        ControlFlow::Continue(Reading { lexing: Lexing::Bare(lexer), words: self })
    }

    fn inwardly<'text>(self, lexer: Lexer<'text, Inside<'text>>) -> Step<'text> {
        ControlFlow::Continue(Reading { lexing: Lexing::Inside(lexer), words: self })
    }

    fn part(self, text: &str, quoting: Quoting, expanding: Expanding) -> Words {
        let begun = self.word.unwrap_or_default();
        Words { word: Some(begun.extended(text, quoting, expanding)), ..self }
    }

    fn redirected<'text>(self, redirect: Redirect, lexer: Lexer<'text, Lexeme<'text>>) -> Step<'text> {
        match self.redirect {
            Some(_pending) => Words::refused(lexer.slice()),
            None => Words { redirect: Some(redirect), ..self }.barely(lexer),
        }
    }

    fn worded(self) -> Words {
        let Words { read, simple, word, redirect } = self;
        match (word, redirect) {
            (None, _) => Words { read, simple, word: None, redirect },
            (Some(_input), Some(Redirect::Input)) => Words { read, simple, word: None, redirect: None },
            (Some(target), Some(Redirect::Output)) => {
                Words { read, simple: simple.writing(target), word: None, redirect: None }
            }
            (Some(argument), None) => Words { read, simple: simple.worded(argument), word: None, redirect: None },
        }
    }

    fn separated(self) -> Result<Words, Unreadable> {
        let Words { read, simple, word, redirect } = self;
        let unaimed = redirect.map(|_pending| Unreadable { span: Redirect::SPAN.to_owned() });
        let keyword = simple.keyword().map(|head| Unreadable { span: head.text.clone() });
        unaimed.or(keyword).map_or(Ok(()), Err)?;
        let kept = (!simple.words.is_empty() || !simple.written.is_empty()).then_some(simple);
        Ok(Words { read: read.into_iter().chain(kept).collect(), simple: Simple::default(), word, redirect: None })
    }

    fn ended(self) -> Result<Vec<Simple>, Unreadable> {
        self.worded().separated().map(|words| words.read)
    }
}

impl Redirect {
    const SPAN: &'static str = "a redirection with no target";
}

impl Simple {
    fn writing(self, target: Word) -> Simple {
        Simple { written: self.written.into_iter().chain([target.text]).collect(), ..self }
    }

    fn worded(self, argument: Word) -> Simple {
        Simple { words: self.words.into_iter().chain([argument]).collect(), ..self }
    }

    fn keyword(&self) -> Option<&Word> {
        let head = self.words.iter().find(|word| Whole::read::<Spelling>(&word.text) != Some(Spelling::Assignment));
        head.filter(|word| {
            word.quoting == Quoting::Plain && Whole::read::<Spelling>(&word.text) == Some(Spelling::Keyword)
        })
    }
}

impl Word {
    fn extended(self, text: &str, quoting: Quoting, expanding: Expanding) -> Word {
        Word {
            text: [self.text.as_str(), text].concat(),
            quoting: self.quoting.max(quoting),
            expanding: self.expanding.max(expanding),
        }
    }
}
