mod boundary;

use design_messages::{OwnerHistory, Quote, Said};
use dispatch_ports::Shell;
use store_ports::Store;
use timer_ports::{Clock, Timeout};
use transcript_ports::Body;

use super::super::crew::{Crew, Prompt};
use crate::cli::failure::Failure;
use boundary::Asked;
pub use boundary::{Answers, Moment};

#[derive(Debug)]
pub struct Answering<'ports, S: Store, C: Clock, G: Shell> {
    store: &'ports S,
    crew: &'ports Crew<'ports, C, G>,
}

impl<'ports, S: Store, C: Clock, G: Shell> Answering<'ports, S, C, G> {
    const TIMEOUT: Timeout = Timeout::new(120);

    pub fn new(store: &'ports S, crew: &'ports Crew<'ports, C, G>) -> Answering<'ports, S, C, G> {
        Answering { store, crew }
    }

    pub fn searched(&self, said: &Body, moment: Moment) -> Result<Option<Answers>, Failure> {
        let asked = Asked::new(said, moment);
        let history: Vec<Said> = self.store.read(OwnerHistory)?;
        let reading = asked.reading(&history);
        let fanned = self.crew.fanned(reading.prompts(), Self::TIMEOUT);
        let quoted: Vec<Quote> = reading.quoted(&fanned).into_iter().flatten().collect();
        let trimmed =
            quoted.first().and_then(|_| self.crew.replied(&Prompt::from(&asked.trimming(&quoted)), Self::TIMEOUT));
        Ok(trimmed.and_then(|reply| Answers::kept(&reply, &quoted)))
    }
}
