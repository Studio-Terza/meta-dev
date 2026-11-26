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
mod stops;
mod structure;
mod subject;
mod verifier;
mod view;

use std::path::PathBuf;
use std::str::FromStr;

use console_ports::Environment;
use design_model::FeatureName;
use design_store::Scope;
use design_store::{Judgments, StopTrail, StreamSchema};
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use grammar_search::Terms;
use store_ports::{Opens, Store};

use super::persistence::Persistence;
use super::{Flag, Ground, Token, Valued};
use crate::cli::failure::Failure;
use crate::{Outcome, Uttered};
pub use apply::{Application, Unresolved};
use console_ports::Shown;
pub use hook::Hook;
use hook::answers::Answering;
use import::Import;
use ingest::Ingest;
use init::Init;
use lifecycle::{Move, Moving, Recast};
use printed::Lines;
use retry::Retry;
use search::Search;
use stops::Trail;
pub use structure::Crew;
use structure::Structure;
use subject::Subject;
use transcript_ports::Body;
use verifier::Verifier;
use view::View;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest(Persistence),
    Show(Subject),
    Search(Terms, Scope),
    Answer(Body),
    Recall(Body),
    Init(Init),
    Apply(PathBuf),
    Move(Move),
    Recast(Recast),
    Structure,
    Stops,
    Hook(Hook),
    View(View),
    Import(PathBuf),
    Retry(Retry),
    Ideals,
    Features,
    Judge(FeatureName),
    Judgments,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Ingest,
    Show,
    Search,
    Answer,
    Recall,
    Init,
    Apply,
    Move(Moving),
    Role,
    Structure,
    Stops,
    Hook,
    View,
    Import,
    Retry,
    Ideals,
    Features,
    Judge,
    Judgments,
}

impl FromStr for Verb {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Verb, Failure> {
        Self::NAMED.iter().find_map(|(known, verb)| (*known == name).then_some(*verb)).ok_or(Failure::Usage)
    }
}

impl Verb {
    const NAMED: [(&'static str, Verb); 22] = [
        ("ingest", Verb::Ingest),
        ("show", Verb::Show),
        ("search", Verb::Search),
        ("answer", Verb::Answer),
        ("recall", Verb::Recall),
        ("init", Verb::Init),
        ("apply", Verb::Apply),
        ("refine", Verb::Move(Moving::Refine)),
        ("specify", Verb::Move(Moving::Specify)),
        ("reopen", Verb::Move(Moving::Reopen)),
        ("drop", Verb::Move(Moving::Drop)),
        ("role", Verb::Role),
        ("structure", Verb::Structure),
        ("stops", Verb::Stops),
        ("hook", Verb::Hook),
        ("view", Verb::View),
        ("import-notes", Verb::Import),
        ("retry", Verb::Retry),
        ("ideals", Verb::Ideals),
        ("features", Verb::Features),
        ("judge", Verb::Judge),
        ("judgments", Verb::Judgments),
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
            | Verb::Answer
            | Verb::Recall
            | Verb::Apply
            | Verb::Move(Moving::Refine | Moving::Reopen)
            | Verb::Role
            | Verb::Structure
            | Verb::Stops
            | Verb::View
            | Verb::Import
            | Verb::Retry
            | Verb::Ideals
            | Verb::Features
            | Verb::Judge
            | Verb::Judgments => &[],
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
            Verb::Answer => Self::spoken(rest).map(Command::Answer),
            Verb::Recall => Self::spoken(rest).map(Command::Recall),
            Verb::Init => Init::try_from(rest).map(Command::Init),
            Verb::Apply => Token::only(rest).map(PathBuf::from).map(Command::Apply),
            Verb::Move(moving) => Move::read(moving, rest).map(Command::Move),
            Verb::Role => Recast::read(rest).map(Command::Recast),
            Verb::Structure => rest.first().map_or(Ok(Command::Structure), |_extra| Err(Failure::Usage)),
            Verb::Stops => rest.first().map_or(Ok(Command::Stops), |_extra| Err(Failure::Usage)),
            Verb::Hook => <&[Token; 1]>::try_from(rest)
                .map_err(|_count| Failure::Usage)
                .and_then(|[token]| token.hook())
                .map(Command::Hook),
            Verb::View => View::read(rest).map(Command::View),
            Verb::Import => Token::only(rest).map(PathBuf::from).map(Command::Import),
            Verb::Retry => Token::only(rest).and_then(str::parse).map(Command::Retry),
            Verb::Ideals => rest.first().map_or(Ok(Command::Ideals), |_extra| Err(Failure::Usage)),
            Verb::Features => rest.first().map_or(Ok(Command::Features), |_extra| Err(Failure::Usage)),
            Verb::Judge => Token::feature(rest).map(Command::Judge),
            Verb::Judgments => rest.first().map_or(Ok(Command::Judgments), |_extra| Err(Failure::Usage)),
        }
    }

    fn spoken(rest: &[Token]) -> Result<Body, Failure> {
        let words = rest.iter().map(Token::text).collect::<Option<Vec<&str>>>().ok_or(Failure::Usage)?;
        words.first().ok_or(Failure::Usage)?;
        Body::try_new(words.join(" ")).map_err(|_malformed| Failure::Usage)
    }
}

impl Command {
    pub fn opened<O: Opens>(&self, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Command::Ingest(persistence) => persistence.opened(path, opens),
            Command::Show(_)
            | Command::Search(..)
            | Command::Answer(_)
            | Command::Recall(_)
            | Command::Init(_)
            | Command::Apply(_)
            | Command::Move(_)
            | Command::Recast(_)
            | Command::Structure
            | Command::Stops
            | Command::Hook(_)
            | Command::View(_)
            | Command::Import(_)
            | Command::Retry(_)
            | Command::Ideals
            | Command::Features
            | Command::Judge(_)
            | Command::Judgments => opens.file::<StreamSchema>(path),
        }
    }

    pub fn written<F: Files, S: Store, D: Dispatch, C: Clock>(
        self,
        environment: &impl Environment,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        match self {
            Command::Ingest(_) => Ingest::read(ground.store(), &ground.transcripts())
                .map(|tally| Uttered::new(Shown::said(Lines::from(&tally.lines())), Ok(Outcome::Ingested))),
            Command::Show(subject) => {
                subject.shown(ground.store()).map(|shown| Uttered::new(shown, Ok(Outcome::Shown)))
            }
            Command::Search(terms, scope) => Search::found(ground.store(), terms, scope)
                .map(|lines| Uttered::new(Shown::said(Lines::from(&lines)), Ok(Outcome::Searched))),
            Command::Answer(said) => Answering::new(ground.store(), crew)
                .searched(&said)
                .map(|searched| Uttered::new(Shown::said(searched), Ok(Outcome::Answered))),
            Command::Recall(said) => Answering::new(ground.store(), crew)
                .recalled(&said)
                .map(|recalled| Uttered::new(Shown::said(recalled), Ok(Outcome::Recalled))),
            Command::Init(init) => init.planted(&ground).map(|initialised| {
                Uttered::new(Shown::said(Lines::from(&initialised.lines())), Ok(Outcome::Initialised))
            }),
            Command::Apply(path) => {
                Application::read(&ground, &path).and_then(|application| application.reported(ground.store()))
            }
            Command::Move(moving) => moving.written(ground.store()),
            Command::Recast(recast) => recast.written(ground.store()),
            Command::Structure => Structure::new(ground.store(), crew).worked().map(structure::Structured::reported),
            Command::Stops => ground
                .store()
                .read(StopTrail)
                .map(|rows| Uttered::new(Shown::said(Trail::from(rows)), Ok(Outcome::Trailed)))
                .map_err(Failure::from),
            Command::Hook(hook) => hook.answered(environment, ground, crew),
            Command::View(view) => view.written(&ground),
            Command::Import(path) => Import::written(&ground, &path),
            Command::Retry(retry) => retry.released(ground.store()),
            Command::Ideals => Verifier::new(&ground, crew).idealised().map(Uttered::from),
            Command::Features => Verifier::new(&ground, crew).featured().map(Uttered::from),
            Command::Judge(name) => Verifier::new(&ground, crew).judged(&name).map(Uttered::from),
            Command::Judgments => ground.store().read(Judgments).map(Uttered::from).map_err(Failure::from),
        }
    }
}
