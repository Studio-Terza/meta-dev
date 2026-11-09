mod apply;
mod hook;
mod import;
mod ingest;
mod init;
mod lifecycle;
mod printed;
mod rendering;
mod retry;
mod search;
mod structure;
mod subject;
mod view;

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use console_ports::Environment;
use design_store::Scope;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use grammar_search::Terms;
use store_ports::{Sqlite, Store};

use super::persistence::Persistence;
use super::{Flag, Ground, Token, Valued};
use crate::Outcome;
use crate::cli::failure::Failure;
pub use apply::{Application, Unresolved};
pub use hook::Hook;
use import::Import;
use ingest::Ingest;
use init::Init;
use lifecycle::{Move, Moving, Recast};
use printed::Printed;
use retry::Retry;
use search::Search;
pub use structure::Crew;
use structure::Structure;
use subject::Subject;
use view::View;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest(Persistence),
    Show(Subject),
    Search(Terms, Scope),
    Init(Init),
    Apply(PathBuf),
    Move(Move),
    Recast(Recast),
    Structure,
    Hook(Hook),
    View(View),
    Import(PathBuf),
    Retry(Retry),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Ingest,
    Show,
    Search,
    Init,
    Apply,
    Move(Moving),
    Role,
    Structure,
    Hook,
    View,
    Import,
    Retry,
}

impl FromStr for Verb {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Verb, Failure> {
        Self::NAMED.iter().find_map(|(known, verb)| (*known == name).then_some(*verb)).ok_or(Failure::Usage)
    }
}

impl Verb {
    const NAMED: [(&'static str, Verb); 15] = [
        ("ingest", Verb::Ingest),
        ("show", Verb::Show),
        ("search", Verb::Search),
        ("init", Verb::Init),
        ("apply", Verb::Apply),
        ("refine", Verb::Move(Moving::Refine)),
        ("specify", Verb::Move(Moving::Specify)),
        ("reopen", Verb::Move(Moving::Reopen)),
        ("drop", Verb::Move(Moving::Drop)),
        ("role", Verb::Role),
        ("structure", Verb::Structure),
        ("hook", Verb::Hook),
        ("view", Verb::View),
        ("import-notes", Verb::Import),
        ("retry", Verb::Retry),
    ];

    pub fn options(self) -> &'static [Flag] {
        match self {
            Verb::Ingest => &[Flag::DryRun],
            Verb::Search => &[Flag::All],
            Verb::Init => &[Flag::Valued(Valued::Consumes)],
            Verb::Move(Moving::Specify) => &[Flag::Valued(Valued::Artifact), Flag::Valued(Valued::Condition)],
            Verb::Move(Moving::Drop) => &[Flag::Valued(Valued::Reason)],
            Verb::Hook => &[Flag::Settings],
            Verb::Show
            | Verb::Apply
            | Verb::Move(Moving::Refine | Moving::Reopen)
            | Verb::Role
            | Verb::Structure
            | Verb::View
            | Verb::Import
            | Verb::Retry => &[],
        }
    }

    pub fn command(self, rest: &[Token], scope: Scope) -> Result<Command, Failure> {
        match self {
            Verb::Ingest => Persistence::try_from(rest).map(Command::Ingest),
            Verb::Show => rest
                .iter()
                .map(Token::text)
                .collect::<Option<Vec<&str>>>()
                .ok_or(Failure::Usage)
                .and_then(|words| words.join(" ").parse())
                .map(Command::Show),
            Verb::Search => rest
                .iter()
                .map(Token::text)
                .collect::<Option<Vec<&str>>>()
                .and_then(Terms::lexed)
                .ok_or(Failure::Usage)
                .map(|terms| Command::Search(terms, scope)),
            Verb::Init => Init::try_from(rest).map(Command::Init),
            Verb::Apply => Token::only(rest).map(PathBuf::from).map(Command::Apply),
            Verb::Move(moving) => Move::read(moving, rest).map(Command::Move),
            Verb::Role => Recast::read(rest).map(Command::Recast),
            Verb::Structure => rest.first().map_or(Ok(Command::Structure), |_extra| Err(Failure::Usage)),
            Verb::Hook => <&[Token; 1]>::try_from(rest)
                .map_err(|_count| Failure::Usage)
                .and_then(|[token]| token.hook())
                .map(Command::Hook),
            Verb::View => View::read(rest).map(Command::View),
            Verb::Import => Token::only(rest).map(PathBuf::from).map(Command::Import),
            Verb::Retry => Token::only(rest).and_then(str::parse).map(Command::Retry),
        }
    }
}

impl Command {
    pub fn opened(&self, path: PathBuf, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        match self {
            Command::Ingest(persistence) => persistence.opened(path, opened),
            Command::Show(_)
            | Command::Search(..)
            | Command::Init(_)
            | Command::Apply(_)
            | Command::Move(_)
            | Command::Recast(_)
            | Command::Structure
            | Command::Hook(_)
            | Command::View(_)
            | Command::Import(_)
            | Command::Retry(_) => opened(path),
        }
    }

    pub fn written<F: Files, S: Store, D: Dispatch, C: Clock>(
        self,
        environment: &impl Environment,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self {
            Command::Ingest(_) => Ingest::read(ground.store(), &ground.transcripts())
                .and_then(|tally| Printed::lines(out, &tally.lines()))
                .map(|_printed| Outcome::Ingested),
            Command::Show(subject) => subject.shown(ground.store(), out).map(|_printed| Outcome::Shown),
            Command::Search(terms, scope) => Search::found(ground.store(), terms, scope)
                .and_then(|lines| Printed::lines(out, &lines))
                .map(|_printed| Outcome::Searched),
            Command::Init(init) => init
                .planted(&ground)
                .and_then(|initialised| Printed::lines(out, &initialised.lines()))
                .map(|_printed| Outcome::Initialised),
            Command::Apply(path) => {
                Application::read(&ground, &path).and_then(|application| application.reported(ground.store(), out))
            }
            Command::Move(moving) => moving.written(ground.store(), out),
            Command::Recast(recast) => recast.written(ground.store(), out),
            Command::Structure => {
                Structure::new(ground.store(), crew).worked().and_then(|structured| structured.reported(out))
            }
            Command::Hook(hook) => hook.answered(environment, ground, crew, out),
            Command::View(view) => view.written(&ground, out),
            Command::Import(path) => Import::written(&ground, &path, out),
            Command::Retry(retry) => retry.released(ground.store(), out),
        }
    }
}
