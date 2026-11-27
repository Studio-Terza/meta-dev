mod boundary;

use design_messages::{OwnerHistory, Quote, Said, Tally};
use dispatch_ports::{Clock, Dispatch, Model, Prompt, Shell, Timeout};
use store_ports::Store;
use transcript_ports::Body;

use super::super::crew::{Crew, Heard};
use crate::cli::failure::Failure;
use boundary::Asked;
pub use boundary::{Answers, Search};

#[derive(Debug)]
pub struct Answering<'ports, S: Store, D: Dispatch, C: Clock, G: Shell> {
    store: &'ports S,
    crew: &'ports Crew<'ports, D, C, G>,
}

impl<'ports, S: Store, D: Dispatch, C: Clock, G: Shell> Answering<'ports, S, D, C, G> {
    const TIMEOUT: Timeout = Timeout::new(120);

    pub fn new(store: &'ports S, crew: &'ports Crew<'ports, D, C, G>) -> Answering<'ports, S, D, C, G> {
        Answering { store, crew }
    }

    pub fn searched(&self, said: &Body) -> Result<Search, Failure> {
        let asked = Asked::from(said);
        let history: Vec<Said> = self.store.read(OwnerHistory)?;
        let reading = asked.reading(&history);
        let fanned = self.crew.fanned(reading.prompts(), Self::TIMEOUT)?;
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
