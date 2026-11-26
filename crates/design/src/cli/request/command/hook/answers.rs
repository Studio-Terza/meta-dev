mod boundary;

use design_store::{ContextNodes, OwnerSaid, Said};
use dispatch_ports::{Clock, Dispatch, Prompt, Timeout};
use store_ports::Store;
use transcript_ports::Body;

use super::super::structure::Crew;
use crate::cli::failure::Failure;
pub use boundary::{Answers, Asked, Recollection, Remembered, Replied, Role, Traced};
use boundary::{Question, Recalling};

#[derive(Debug)]
pub struct Answering<'ports, S: Store, D: Dispatch, C: Clock> {
    store: &'ports S,
    crew: &'ports Crew<'ports, D, C>,
    role: Role,
}

impl<'ports, S: Store, D: Dispatch, C: Clock> Answering<'ports, S, D, C> {
    const TIMEOUT: Timeout = Timeout::new(120);
    const RECALLING: Timeout = Timeout::new(45);

    pub fn new(store: &'ports S, crew: &'ports Crew<'ports, D, C>) -> Answering<'ports, S, D, C> {
        Answering { store, crew, role: Role::LEAD }
    }

    pub fn answered(&self, said: &Body) -> Result<Option<Answers>, Failure> {
        self.traced(said).map(Traced::answers)
    }

    pub fn recalled(&self, said: &Body) -> Result<Recollection, Failure> {
        let nodes = self.store.read(ContextNodes)?;
        let replied = self.crew.replied(&Prompt::from(&Recalling::new(said, &nodes)), Self::RECALLING)?;
        Ok(replied.map_or_default(|reply| Recollection::recalled(&reply, &nodes)))
    }

    pub fn traced(&self, said: &Body) -> Result<Traced, Failure> {
        let classified = self.crew.replied(&Prompt::from(&self.role.classifying(said)), Self::TIMEOUT)?;
        classified.as_ref().and_then(Asked::read).map_or(Ok(Traced::Statement), |asked| self.selected(asked))
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
