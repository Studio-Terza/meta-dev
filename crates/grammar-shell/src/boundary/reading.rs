use std::iter;
use std::ops::ControlFlow;

use grammar::logos::{Lexer, Logos};
use grammar::{Ignoring, Unlexed};

use super::lexeme::{Argued, Lexeme, Line, PerlFlag, Quoted, SedFlag, Spelling, Whole};
use super::occurrence::{Changed, Construct, Edit, Editor, Flagged, Occurrence, Spotted};

enum Lexing<'text> {
    Command(Lexer<'text, Lexeme<'text>>),
    Quoted(Lexer<'text, Quoted<'text>>),
    Body(Lexer<'text, Line<'text>>, Body<'text>),
}

#[derive(Debug)]
struct Body<'text> {
    pending: Pending,
    waiting: Vec<Pending>,
    start: usize,
    line: usize,
    seen: Seen<'text>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Seen<'text> {
    Nothing,
    Tabs,
    Text(&'text str),
    Indented(&'text str),
}

pub(super) struct Reading<'text> {
    lexing: Lexing<'text>,
    state: State,
}

#[derive(Debug)]
struct State {
    top: Frame,
    outer: Vec<Frame>,
    heredocs: Vec<Pending>,
    found: Vec<Occurrence>,
}

#[derive(Debug)]
struct Frame {
    construct: Option<Construct>,
    closer: Closer,
    resume: Mode,
    simple: Simple,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Closer {
    Top,
    Paren,
    Backtick,
    Arithmetic,
    Done,
    Fi,
    Esac,
    Brace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Command,
    Quoted,
}

#[derive(Debug)]
struct Simple {
    word: Option<Word>,
    position: Position,
    prefixes: Vec<Construct>,
    redirect: Option<Redirect>,
    moved: Option<Vec<String>>,
    cd: Option<Cding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cding;

#[derive(Debug, Default)]
struct Word {
    text: String,
    quoting: Quoting,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Quoting {
    #[default]
    Plain,
    Quoted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Redirect {
    Input,
    Output,
    Delimiter(Strip),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strip {
    Tabs,
    Kept,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Position {
    Command,
    Argument(Head),
    Pattern,
    Named,
    Body,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Head {
    Plain,
    Shell(Script),
    Xargs(Expecting),
    Find,
    Case,
    Sed,
    Perl,
    Tee,
    Mv,
    Cd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    Options,
    Awaiting,
    Given,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expecting {
    Utility,
    Value,
}

#[derive(Debug)]
struct Pending {
    delimiter: String,
    strip: Strip,
    fed: Option<Vec<Construct>>,
}

impl<'text> Reading<'text> {
    pub(super) fn occurrences(text: &'text str) -> Vec<Occurrence> {
        let first = Reading { lexing: Lexing::Command(Lexeme::lexer(text)), state: State::new() };
        match iter::repeat(()).try_fold(first, |reading, ()| reading.step()) {
            ControlFlow::Break(found) => found,
            ControlFlow::Continue(_endless) => Vec::new(),
        }
    }

    fn step(self) -> ControlFlow<Vec<Occurrence>, Reading<'text>> {
        let Reading { lexing, state } = self;
        match lexing {
            Lexing::Command(mut lexer) => match lexer.next() {
                Some(Ok(lexeme)) => ControlFlow::Continue(state.commanded(lexeme, lexer)),
                Some(Err(())) => ControlFlow::Continue(state.noted(Spotted::Unreadable).commanding(lexer)),
                None => ControlFlow::Break(state.ended()),
            },
            Lexing::Quoted(mut lexer) => match lexer.next() {
                Some(Ok(lexeme)) => ControlFlow::Continue(state.quoted(lexeme, lexer)),
                Some(Err(())) => ControlFlow::Continue(state.noted(Spotted::Unreadable).quoting(lexer)),
                None => ControlFlow::Break(state.ended()),
            },
            Lexing::Body(mut lexer, body) => {
                match lexer.next().and_then(|lexed| lexed.ignoring(Unlexed::NotThisGrammar)) {
                    Some(line) => ControlFlow::Continue(state.bodied(line, lexer, body)),
                    None => ControlFlow::Break(state.unterminated(&lexer, &body)),
                }
            }
        }
    }
}

impl State {
    fn new() -> State {
        State {
            top: Frame::new(None, Closer::Top, Mode::Command, Position::Command),
            outer: Vec::new(),
            heredocs: Vec::new(),
            found: Vec::new(),
        }
    }

    fn ended(self) -> Vec<Occurrence> {
        self.worded().flushed().found
    }

    fn commanded<'text>(self, lexeme: Lexeme<'text>, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        match lexeme {
            Lexeme::Literal(text) => self.part(text, Quoting::Plain).commanding(lexer),
            Lexeme::Single(text) | Lexeme::Escaped(text) | Lexeme::Expansion(text) => {
                self.part(text, Quoting::Quoted).commanding(lexer)
            }
            Lexeme::Quote => self.part("", Quoting::Quoted).quoting(lexer.morph()),
            Lexeme::Unreadable => self.worded().noted(Spotted::Unreadable).commanding(lexer),
            Lexeme::Substitution => self.substituted(Closer::Paren, Mode::Command).commanding(lexer),
            Lexeme::Arithmetic => self.counted(Mode::Command).commanding(lexer),
            Lexeme::Backtick => self.backticked(lexer),
            Lexeme::Blank | Lexeme::Comment | Lexeme::Duplicate => self.worded().commanding(lexer),
            Lexeme::Newline => self.worded().separated().heredocs(lexer),
            Lexeme::Separator => self.worded().separated().commanding(lexer),
            Lexeme::CaseEnd => self.worded().case_ended().commanding(lexer),
            Lexeme::Open => self.opened().commanding(lexer),
            Lexeme::Parens => self.worded().positioned(Position::Body).commanding(lexer),
            Lexeme::Close => self.worded().closed(lexer),
            Lexeme::Closes => self.worded().closes(lexer),
            Lexeme::Heredoc => self.worded().redirected(Redirect::Delimiter(Strip::Kept)).commanding(lexer),
            Lexeme::Stripped => self.worded().redirected(Redirect::Delimiter(Strip::Tabs)).commanding(lexer),
            Lexeme::Input => self.worded().redirected(Redirect::Input).commanding(lexer),
            Lexeme::Output => self.worded().redirected(Redirect::Output).commanding(lexer),
        }
    }

    fn quoted<'text>(self, lexeme: Quoted<'text>, lexer: Lexer<'text, Quoted<'text>>) -> Reading<'text> {
        match lexeme {
            Quoted::End => self.commanding(lexer.morph()),
            Quoted::Unreadable => self.noted(Spotted::Unreadable).quoting(lexer),
            Quoted::Text(text) | Quoted::Escaped(text) => self.part(text, Quoting::Quoted).quoting(lexer),
            Quoted::Substitution => self.substituted(Closer::Paren, Mode::Quoted).commanding(lexer.morph()),
            Quoted::Backtick => self.substituted(Closer::Backtick, Mode::Quoted).commanding(lexer.morph()),
            Quoted::Arithmetic => self.counted(Mode::Quoted).commanding(lexer.morph()),
        }
    }

    fn commanding<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        Reading { lexing: Lexing::Command(lexer), state: self }
    }

    fn quoting<'text>(self, lexer: Lexer<'text, Quoted<'text>>) -> Reading<'text> {
        Reading { lexing: Lexing::Quoted(lexer), state: self }
    }

    fn substituted(self, closer: Closer, resume: Mode) -> State {
        let frame = Frame::new(Some(Construct::Substitution), closer, resume, Position::Command);
        self.part("", Quoting::Quoted).pushed(frame)
    }

    fn counted(self, resume: Mode) -> State {
        let frame = Frame::new(None, Closer::Arithmetic, resume, Position::Argument(Head::Plain));
        self.part("", Quoting::Quoted).pushed(frame)
    }

    fn backticked<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        match self.top.closer {
            Closer::Backtick => self.worded().popped(lexer),
            Closer::Top
            | Closer::Paren
            | Closer::Arithmetic
            | Closer::Done
            | Closer::Fi
            | Closer::Esac
            | Closer::Brace => self.substituted(Closer::Backtick, Mode::Command).commanding(lexer),
        }
    }

    fn popped<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        let resume = self.top.resume;
        let state = self.unwound();
        match resume {
            Mode::Command => state.commanding(lexer),
            Mode::Quoted => state.quoting(lexer.morph()),
        }
    }

    fn unwound(self) -> State {
        let State { top, mut outer, heredocs, found } = self.flushed();
        match outer.pop() {
            Some(parent) => State { top: parent, outer, heredocs, found },
            None => State { top, outer, heredocs, found },
        }
    }

    fn closed<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        match (self.top.simple.position, self.top.closer) {
            (Position::Pattern, _) => self.positioned(Position::Command).commanding(lexer),
            (Position::Command | Position::Argument(_) | Position::Named | Position::Body, closer) => match closer {
                Closer::Paren | Closer::Arithmetic => self.popped(lexer),
                Closer::Top | Closer::Backtick | Closer::Done | Closer::Fi | Closer::Esac | Closer::Brace => {
                    self.commanding(lexer)
                }
            },
        }
    }

    fn closes<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        match self.top.closer {
            Closer::Arithmetic => self.popped(lexer),
            Closer::Paren => self.unwound().closed(lexer),
            Closer::Top | Closer::Backtick | Closer::Done | Closer::Fi | Closer::Esac | Closer::Brace => {
                self.commanding(lexer)
            }
        }
    }

    fn opened(self) -> State {
        match (self.top.simple.word.as_ref(), self.top.simple.position) {
            (Some(_word), _) => {
                self.pushed(Frame::new(None, Closer::Paren, Mode::Command, Position::Argument(Head::Plain)))
            }
            (None, Position::Body) => {
                self.pushed(Frame::new(Some(Construct::Function), Closer::Paren, Mode::Command, Position::Command))
            }
            (None, Position::Pattern) => self,
            (None, Position::Command | Position::Argument(_) | Position::Named) => self
                .positioned(Position::Argument(Head::Plain))
                .pushed(Frame::new(Some(Construct::Subshell), Closer::Paren, Mode::Command, Position::Command)),
        }
    }

    fn pushed(self, frame: Frame) -> State {
        let State { top, mut outer, heredocs, found } = self;
        outer.push(top);
        State { top: frame, outer, heredocs, found }
    }

    fn opening(self, construct: Construct, closer: Closer, inner: Position) -> State {
        let frame = Frame::new(Some(construct), closer, Mode::Command, inner);
        self.positioned(Position::Argument(Head::Plain)).pushed(frame)
    }

    fn ended_by(self, closer: Closer) -> State {
        if self.top.closer == closer { self.unwound() } else { self }
    }

    fn simpled(self, change: impl FnOnce(Simple) -> Simple) -> State {
        let State { top, outer, heredocs, found } = self;
        State { top: Frame { simple: change(top.simple), ..top }, outer, heredocs, found }
    }

    fn positioned(self, position: Position) -> State {
        self.simpled(|simple| Simple { position, ..simple })
    }

    fn redirected(self, redirect: Redirect) -> State {
        self.simpled(|simple| Simple { redirect: Some(redirect), ..simple })
    }

    fn prefixed(self, construct: Construct) -> State {
        self.simpled(|simple| Simple {
            prefixes: simple.prefixes.into_iter().chain(iter::once(construct)).collect(),
            position: Position::Command,
            ..simple
        })
    }

    fn part(self, text: &str, quoting: Quoting) -> State {
        self.simpled(|simple| Simple { word: Some(simple.word.unwrap_or_default().extended(text, quoting)), ..simple })
    }

    fn separated(self) -> State {
        self.flushed().simpled(Simple::separated)
    }

    fn flushed(self) -> State {
        let State { top, outer, heredocs, found } = self;
        let Frame { construct, closer, resume, simple } = top;
        let Simple { word, position, prefixes, redirect, moved, cd } = simple;
        let state = State {
            top: Frame {
                construct,
                closer,
                resume,
                simple: Simple { word, position, prefixes, redirect, moved: None, cd: None },
            },
            outer,
            heredocs,
            found,
        };
        let spoken = moved.as_deref().and_then(<[String]>::split_last);
        let edit = spoken.filter(|(_destination, sources)| !sources.is_empty()).map(|(destination, sources)| {
            Spotted::Edit(Edit::Moved { sources: sources.to_vec(), destination: destination.clone() })
        });
        let unknown = cd.map(|Cding| Spotted::Changed(Changed::Unknown));
        edit.into_iter().chain(unknown).fold(state, State::noted)
    }

    fn spotted(self, edit: Edit) -> State {
        self.noted(Spotted::Edit(edit))
    }

    fn noted(self, spotted: Spotted) -> State {
        let occurrence = Occurrence::new(spotted, self.within());
        State { found: self.found.into_iter().chain(iter::once(occurrence)).collect(), ..self }
    }

    fn case_ended(self) -> State {
        match self.top.closer {
            Closer::Esac => self.separated().positioned(Position::Pattern),
            Closer::Top
            | Closer::Paren
            | Closer::Backtick
            | Closer::Arithmetic
            | Closer::Done
            | Closer::Fi
            | Closer::Brace => self.separated(),
        }
    }

    fn worded(self) -> State {
        let State { top, outer, heredocs, found } = self;
        let Frame { construct, closer, resume, simple: building } = top;
        let Simple { word: taken, position, prefixes, redirect, moved, cd } = building;
        let simple = Simple { word: None, position, prefixes, redirect, moved, cd };
        let state = State { top: Frame { construct, closer, resume, simple }, outer, heredocs, found };
        match taken {
            Some(word) => state.word(word),
            None => state,
        }
    }

    fn word(self, word: Word) -> State {
        match self.top.simple.redirect {
            Some(Redirect::Input) => self.simpled(Simple::redirected),
            Some(Redirect::Output) => self.spotted(Edit::Wrote(word.text)).simpled(Simple::redirected),
            Some(Redirect::Delimiter(strip)) => {
                let fed = self.fed();
                self.simpled(Simple::redirected).awaiting(Pending { delimiter: word.text, strip, fed })
            }
            None => self.terminated(&word),
        }
    }

    fn fed(&self) -> Option<Vec<Construct>> {
        match self.top.simple.position {
            Position::Argument(Head::Shell(_)) => Some(self.within()),
            Position::Command
            | Position::Argument(
                Head::Plain
                | Head::Xargs(_)
                | Head::Find
                | Head::Case
                | Head::Sed
                | Head::Perl
                | Head::Tee
                | Head::Mv
                | Head::Cd,
            )
            | Position::Pattern
            | Position::Named
            | Position::Body => None,
        }
    }

    fn awaiting(self, pending: Pending) -> State {
        State { heredocs: self.heredocs.into_iter().chain(iter::once(pending)).collect(), ..self }
    }

    fn terminated(self, word: &Word) -> State {
        let execing = self.top.simple.prefixes.last() == Some(&Construct::Find);
        match (execing, Whole::read::<Argued>(&word.text)) {
            (true, Some(Argued::Terminator)) => self.simpled(Simple::unprefixed),
            (true | false, _) => self.placed(word),
        }
    }

    fn placed(self, word: &Word) -> State {
        let spelled = (Whole::read::<Spelling>(&word.text), word.quoting);
        match self.top.simple.position {
            Position::Command => self.command(word),
            Position::Argument(head) => self.argued(head, word),
            Position::Pattern => match spelled {
                (Some(Spelling::Esac), Quoting::Plain) => self.ended_by(Closer::Esac),
                (_, Quoting::Plain | Quoting::Quoted) => self,
            },
            Position::Named => self.positioned(Position::Body),
            Position::Body => match spelled {
                (Some(Spelling::Brace), Quoting::Plain) => {
                    self.opening(Construct::Function, Closer::Brace, Position::Command)
                }
                (_, Quoting::Plain | Quoting::Quoted) => self.command(word),
            },
        }
    }

    fn command(self, word: &Word) -> State {
        let argument = |head| Position::Argument(head);
        match (Whole::read::<Spelling>(&word.text), word.quoting) {
            (Some(Spelling::For), Quoting::Plain) => self.opening(Construct::Loop, Closer::Done, argument(Head::Plain)),
            (Some(Spelling::While), Quoting::Plain) => self.opening(Construct::Loop, Closer::Done, Position::Command),
            (Some(Spelling::If), Quoting::Plain) => self.opening(Construct::Conditional, Closer::Fi, Position::Command),
            (Some(Spelling::Case), Quoting::Plain) => {
                self.opening(Construct::Conditional, Closer::Esac, argument(Head::Case))
            }
            (Some(Spelling::Lead), Quoting::Plain) | (Some(Spelling::Assignment), _) => self,
            (Some(Spelling::Brace), Quoting::Plain) => self.opening(Construct::Group, Closer::Brace, Position::Command),
            (Some(Spelling::Unbrace), Quoting::Plain) => self.ended_by(Closer::Brace),
            (Some(Spelling::Done), Quoting::Plain) => self.ended_by(Closer::Done),
            (Some(Spelling::Fi), Quoting::Plain) => self.ended_by(Closer::Fi),
            (Some(Spelling::Esac), Quoting::Plain) => self.ended_by(Closer::Esac),
            (Some(Spelling::Function), Quoting::Plain) => self.positioned(Position::Named),
            (Some(Spelling::Rm), _) => self.flagged(Flagged::Rm, None).positioned(argument(Head::Plain)),
            (Some(Spelling::Rmdir), _) => self.flagged(Flagged::Rmdir, None).positioned(argument(Head::Plain)),
            (Some(Spelling::Shell), _) => self.positioned(argument(Head::Shell(Script::Options))),
            (Some(Spelling::Xargs), _) => self.positioned(argument(Head::Xargs(Expecting::Utility))),
            (Some(Spelling::Find), _) => self.positioned(argument(Head::Find)),
            (Some(Spelling::Sed), _) => self.positioned(argument(Head::Sed)),
            (Some(Spelling::Perl), _) => self.positioned(argument(Head::Perl)),
            (Some(Spelling::Tee), _) => self.positioned(argument(Head::Tee)),
            (Some(Spelling::Mv), _) => {
                self.simpled(|simple| Simple { moved: Some(Vec::new()), ..simple }).positioned(argument(Head::Mv))
            }
            (Some(Spelling::Cd), _) => {
                self.simpled(|simple| Simple { cd: Some(Cding), ..simple }).positioned(argument(Head::Cd))
            }
            (
                Some(
                    Spelling::For
                    | Spelling::While
                    | Spelling::If
                    | Spelling::Case
                    | Spelling::Lead
                    | Spelling::Brace
                    | Spelling::Unbrace
                    | Spelling::Done
                    | Spelling::Fi
                    | Spelling::Esac
                    | Spelling::Function
                    | Spelling::Test,
                ),
                Quoting::Quoted,
            )
            | (Some(Spelling::Test), Quoting::Plain)
            | (None, _) => self.positioned(argument(Head::Plain)),
        }
    }

    fn argued(self, head: Head, word: &Word) -> State {
        match head {
            Head::Sed => match Whole::read::<SedFlag>(&word.text) {
                Some(SedFlag::InPlace) => self.spotted(Edit::InPlace(Editor::Sed)),
                Some(SedFlag::Other) | None => self,
            },
            Head::Perl => match Whole::read::<PerlFlag>(&word.text) {
                Some(PerlFlag::InPlace) => self.spotted(Edit::InPlace(Editor::Perl)),
                Some(PerlFlag::Other) | None => self,
            },
            Head::Tee => match Whole::read::<PerlFlag>(&word.text) {
                Some(PerlFlag::InPlace | PerlFlag::Other) => self,
                None => self.spotted(Edit::Wrote(word.text.clone())),
            },
            Head::Mv => match Whole::read::<PerlFlag>(&word.text) {
                Some(PerlFlag::InPlace | PerlFlag::Other) => self,
                None => self.simpled(|simple| Simple {
                    moved: simple.moved.map(|moved| moved.into_iter().chain(iter::once(word.text.clone())).collect()),
                    ..simple
                }),
            },
            Head::Cd => match Whole::read::<PerlFlag>(&word.text) {
                Some(PerlFlag::InPlace | PerlFlag::Other) => self,
                None => {
                    let opaque = word.text.chars().any(|letter| Self::OPAQUE.contains(&letter));
                    let changed = if opaque { Changed::Unknown } else { Changed::To(word.text.clone()) };
                    self.simpled(|simple| Simple { cd: None, ..simple })
                        .noted(Spotted::Changed(changed))
                        .positioned(Position::Argument(Head::Plain))
                }
            },
            Head::Plain | Head::Shell(_) | Head::Xargs(_) | Head::Find | Head::Case => {
                self.scanned_argument(head, word)
            }
        }
    }

    const OPAQUE: [char; 3] = ['$', '`', '~'];

    fn scanned_argument(self, head: Head, word: &Word) -> State {
        let argued = Whole::read::<Argued>(&word.text);
        let shell = |script| Position::Argument(Head::Shell(script));
        match (head, argued) {
            (
                Head::Plain | Head::Shell(Script::Given) | Head::Sed | Head::Perl | Head::Tee | Head::Mv | Head::Cd,
                _,
            )
            | (
                Head::Case,
                Some(
                    Argued::Exec | Argued::Delete | Argued::Terminator | Argued::Valued | Argued::Script | Argued::Flag,
                )
                | None,
            )
            | (Head::Shell(Script::Options), Some(Argued::Flag | Argued::Valued | Argued::Exec | Argued::Delete))
            | (Head::Xargs(Expecting::Utility), Some(Argued::Flag | Argued::Script | Argued::Exec | Argued::Delete))
            | (
                Head::Find,
                Some(Argued::Terminator | Argued::In | Argued::Valued | Argued::Script | Argued::Flag) | None,
            ) => self,
            (Head::Case, Some(Argued::In)) => self.positioned(Position::Pattern),
            (Head::Shell(Script::Options), Some(Argued::Script)) => self.positioned(shell(Script::Awaiting)),
            (Head::Shell(Script::Options), Some(Argued::Terminator | Argued::In) | None) => {
                self.positioned(shell(Script::Given))
            }
            (Head::Shell(Script::Awaiting), _) => self.scripted(&word.text).positioned(shell(Script::Given)),
            (Head::Xargs(Expecting::Value), _) => self.positioned(Position::Argument(Head::Xargs(Expecting::Utility))),
            (Head::Xargs(Expecting::Utility), Some(Argued::Valued)) => {
                self.positioned(Position::Argument(Head::Xargs(Expecting::Value)))
            }
            (Head::Xargs(Expecting::Utility), Some(Argued::Terminator | Argued::In) | None) => {
                self.prefixed(Construct::Xargs).command(word)
            }
            (Head::Find, Some(Argued::Exec)) => self.prefixed(Construct::Find),
            (Head::Find, Some(Argued::Delete)) => self.flagged(Flagged::Delete, Some(Construct::Find)),
        }
    }

    fn within(&self) -> Vec<Construct> {
        self.outer
            .iter()
            .chain(iter::once(&self.top))
            .filter_map(|frame| frame.construct)
            .chain(self.top.simple.prefixes.iter().copied())
            .collect()
    }

    fn flagged(self, flagged: Flagged, construct: Option<Construct>) -> State {
        let within = self.within().into_iter().chain(construct).collect();
        let occurrence = Occurrence::new(Spotted::Flagged(flagged), within);
        State { found: self.found.into_iter().chain(iter::once(occurrence)).collect(), ..self }
    }

    fn scripted(self, script: &str) -> State {
        let outer: Vec<Construct> = self.within().into_iter().chain(iter::once(Construct::ShellScript)).collect();
        let inner = Reading::occurrences(script).into_iter().map(|occurrence| occurrence.inside(&outer));
        State { found: self.found.into_iter().chain(inner).collect(), ..self }
    }

    fn heredocs<'text>(self, lexer: Lexer<'text, Lexeme<'text>>) -> Reading<'text> {
        let State { top, outer, heredocs, found } = self;
        let state = State { top, outer, heredocs: Vec::new(), found };
        let mut waiting: Vec<Pending> = heredocs.into_iter().rev().collect();
        match waiting.pop() {
            Some(pending) => {
                let start = lexer.span().end;
                state.bodying(lexer.morph(), Body { pending, waiting, start, line: start, seen: Seen::Nothing })
            }
            None => state.commanding(lexer),
        }
    }

    fn bodied<'text>(self, line: Line<'text>, lexer: Lexer<'text, Line<'text>>, body: Body<'text>) -> Reading<'text> {
        match line {
            Line::Tabs => self.bodying(lexer, Body { seen: body.seen.tabbed(), ..body }),
            Line::Text(text) => self.bodying(lexer, Body { seen: body.seen.texted(text), ..body }),
            Line::End => {
                let end = lexer.span().end;
                match body.read().filter(|read| *read == body.pending.delimiter) {
                    Some(_delimiter) => {
                        let text = lexer.source().get(body.start..body.line).unwrap_or_default();
                        let Body { pending, mut waiting, .. } = body;
                        let state = self.scanned(&pending, text);
                        match waiting.pop() {
                            Some(next) => state.bodying(
                                lexer,
                                Body { pending: next, waiting, start: end, line: end, seen: Seen::Nothing },
                            ),
                            None => state.commanding(lexer.morph()),
                        }
                    }
                    None => self.bodying(lexer, Body { line: end, seen: Seen::Nothing, ..body }),
                }
            }
        }
    }

    fn unterminated<'text>(self, lexer: &Lexer<'text, Line<'text>>, body: &Body<'text>) -> Vec<Occurrence> {
        let text = lexer.source().get(body.start..).unwrap_or_default();
        self.scanned(&body.pending, text).ended()
    }

    fn bodying<'text>(self, lexer: Lexer<'text, Line<'text>>, body: Body<'text>) -> Reading<'text> {
        Reading { lexing: Lexing::Body(lexer, body), state: self }
    }

    fn scanned(self, pending: &Pending, text: &str) -> State {
        State { found: self.found.into_iter().chain(pending.read(text)).collect(), ..self }
    }
}

impl<'text> Body<'text> {
    fn read(&self) -> Option<&'text str> {
        match (self.seen, self.pending.strip) {
            (Seen::Nothing, Strip::Tabs | Strip::Kept) | (Seen::Tabs, Strip::Tabs) => Some(""),
            (Seen::Text(text), Strip::Tabs | Strip::Kept) | (Seen::Indented(text), Strip::Tabs) => Some(text),
            (Seen::Tabs | Seen::Indented(_), Strip::Kept) => None,
        }
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

impl Frame {
    fn new(construct: Option<Construct>, closer: Closer, resume: Mode, position: Position) -> Frame {
        Frame {
            construct,
            closer,
            resume,
            simple: Simple { word: None, position, prefixes: Vec::new(), redirect: None, moved: None, cd: None },
        }
    }
}

impl Simple {
    fn separated(self) -> Simple {
        let position = match self.position {
            Position::Pattern => Position::Pattern,
            Position::Command | Position::Argument(_) | Position::Named | Position::Body => Position::Command,
        };
        Simple { word: None, position, prefixes: Vec::new(), redirect: None, moved: None, cd: None }
    }

    fn redirected(self) -> Simple {
        Simple { redirect: None, ..self }
    }

    fn unprefixed(self) -> Simple {
        let Simple { word, prefixes, redirect, moved, cd, .. } = self;
        let kept = prefixes.len().saturating_sub(1);
        Simple {
            word,
            position: Position::Argument(Head::Find),
            prefixes: prefixes.into_iter().take(kept).collect(),
            redirect,
            moved,
            cd,
        }
    }
}

impl Word {
    fn extended(self, text: &str, quoting: Quoting) -> Word {
        Word { text: [self.text.as_str(), text].concat(), quoting: self.quoting.max(quoting) }
    }
}

impl Pending {
    fn read(&self, body: &str) -> Vec<Occurrence> {
        self.fed.as_ref().map_or_else(Vec::new, |fed| {
            let outer: Vec<Construct> = fed.iter().copied().chain(iter::once(Construct::Heredoc)).collect();
            Reading::occurrences(body).into_iter().map(|occurrence| occurrence.inside(&outer)).collect()
        })
    }
}
