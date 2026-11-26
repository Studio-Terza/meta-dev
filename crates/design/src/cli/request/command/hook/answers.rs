mod boundary;

use design_store::{OwnerSaid, Said};
use dispatch_ports::{Clock, Dispatch, Prompt, Timeout};
use store_ports::Store;
use transcript_ports::Body;

use super::super::structure::Crew;
use crate::cli::failure::Failure;
pub use boundary::{Answers, Traced};
use boundary::{Asked, Question, Role};

#[derive(Debug)]
pub struct Answering<'ports, S: Store, D: Dispatch, C: Clock> {
    store: &'ports S,
    crew: &'ports Crew<'ports, D, C>,
    role: Role,
}

impl<'ports, S: Store, D: Dispatch, C: Clock> Answering<'ports, S, D, C> {
    const TIMEOUT: Timeout = Timeout::new(120);

    pub fn new(store: &'ports S, crew: &'ports Crew<'ports, D, C>) -> Answering<'ports, S, D, C> {
        Answering { store, crew, role: Role::LEAD }
    }

    pub fn answered(&self, said: &Body) -> Result<Option<Answers>, Failure> {
        self.traced(said).map(Traced::answers)
    }

    pub fn traced(&self, said: &Body) -> Result<Traced, Failure> {
        let classified = self.crew.replied(&Prompt::from(&self.role.classifying(said)), Self::TIMEOUT)?;
        classified.and_then(Asked::read).map_or(Ok(Traced::Statement), |asked| self.selected(asked))
    }

    fn selected(&self, asked: Asked) -> Result<Traced, Failure> {
        let candidates: Vec<Said> =
            asked.terms().map(|terms| self.store.read(OwnerSaid::any(terms))).transpose()?.unwrap_or_default();
        let picked = candidates
            .first()
            .map(|_| self.crew.replied(&Prompt::from(&asked.selecting(&candidates)), Self::TIMEOUT))
            .transpose()?
            .flatten();
        let answers = picked.and_then(|reply| Answers::verified(&reply, &candidates));
        Ok(Traced::Question(Question::new(asked, candidates, answers)))
    }
}
