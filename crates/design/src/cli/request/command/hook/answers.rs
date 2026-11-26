mod boundary;

use design_store::{ContextNodes, OwnerHistory, Quote, Said, Tally};
use dispatch_ports::{Clock, Dispatch, Model, Prompt, Timeout};
use store_ports::Store;
use transcript_ports::Body;

use super::super::structure::{Crew, Heard};
use crate::cli::failure::Failure;
use boundary::Recalling;
pub use boundary::{Answers, Asked, Reading, Recollection, Remembered, Search};

#[derive(Debug)]
pub struct Answering<'ports, S: Store, D: Dispatch, C: Clock> {
    store: &'ports S,
    crew: &'ports Crew<'ports, D, C>,
}

impl<'ports, S: Store, D: Dispatch, C: Clock> Answering<'ports, S, D, C> {
    const TIMEOUT: Timeout = Timeout::new(120);
    const RECALLING: Timeout = Timeout::new(45);

    pub fn new(store: &'ports S, crew: &'ports Crew<'ports, D, C>) -> Answering<'ports, S, D, C> {
        Answering { store, crew }
    }

    pub fn recalled(&self, said: &Body) -> Result<Recollection, Failure> {
        let nodes = self.store.read(ContextNodes)?;
        let heard = self.crew.replied(&Prompt::from(&Recalling::new(said, &nodes)), Self::RECALLING, Model::Haiku)?;
        Ok(heard.reply().map_or_default(|reply| Recollection::recalled(reply, &nodes)))
    }

    pub fn searched(&self, said: &Body) -> Result<Search, Failure> {
        let asked = Asked::from(said);
        let history: Vec<Said> = self.store.read(OwnerHistory)?;
        let reading = asked.reading(&history);
        let fanned = self.crew.fanned(&reading, Self::TIMEOUT)?;
        let quoted: Vec<Quote> = reading.quoted(&fanned).into_iter().flatten().collect();
        let trimmed = quoted
            .first()
            .map(|_| self.crew.replied(&Prompt::from(&asked.trimming(&quoted)), Self::TIMEOUT, Model::Haiku))
            .transpose()?;
        let answers = trimmed.as_ref().and_then(Heard::reply).and_then(|reply| Answers::kept(reply, &quoted));
        let tokens: Tally = fanned.iter().chain(trimmed.iter()).map(Heard::tokens).sum();
        Ok(Search::new(&history, &quoted, answers, tokens))
    }
}
