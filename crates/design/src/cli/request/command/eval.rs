mod boundary;

use design_store::{OwnerBefore, OwnerSaid, Said, TurnEnd, Unabridged};
use dispatch_ports::{Clock, Dispatch, Model, Prompt, Timeout};
use store_ports::Store;
use transcript_ports::{Body, Instant};

use algebra::Monoid;

use super::hook::answers::{Asked, Role};
use super::structure::Crew;
use crate::cli::failure::Failure;
use boundary::{Arm, Grounding, Judged, Truth, Turn};
pub use boundary::{Called, Eval, Evaluated};

#[derive(Debug)]
struct Evaluating<'ports, S: Store, D: Dispatch, C: Clock> {
    store: &'ports S,
    crew: &'ports Crew<'ports, D, C>,
}

impl Eval {
    pub fn evaluated<S: Store, D: Dispatch, C: Clock>(
        &self,
        store: &S,
        crew: &Crew<'_, D, C>,
    ) -> Result<Evaluated, Failure> {
        let ends = store.read(self.turn_ends())?;
        let evaluating = Evaluating { store, crew };
        ends.iter().map(|end| evaluating.turn(end)).collect::<Result<Vec<Turn>, Failure>>().map(Evaluated::from)
    }
}

impl<S: Store, D: Dispatch, C: Clock> Evaluating<'_, S, D, C> {
    const TIMEOUT: Timeout = Timeout::new(600);
    const CHUNK: usize = 300;

    fn turn(&self, end: &TurnEnd) -> Result<Turn, Failure> {
        let said: &Body = end.asked().as_ref();
        let classifying = Prompt::from(&Role::LEAD.classifying(said));
        let haiku = self.crew.called(&classifying, Self::TIMEOUT, Model::Haiku)?;
        let sonnet = self.crew.called(&classifying, Self::TIMEOUT, Model::Sonnet)?;
        let asked = haiku.reply().and_then(Asked::read);
        let truly = sonnet.reply().and_then(Asked::read);
        let judged = asked
            .as_ref()
            .zip(truly.as_ref())
            .map(|(question, truth)| self.judged(end, question, truth))
            .transpose()?;
        Ok(Turn::new(end, [asked.as_ref(), truly.as_ref()], judged))
    }

    fn judged(&self, end: &TurnEnd, asked: &Asked, truly: &Asked) -> Result<Judged, Failure> {
        let at: &Instant = end.asked().as_ref();
        let history: Vec<Said> = self.store.read(OwnerBefore::from(at.clone()))?;
        let candidates: Vec<Said> = asked
            .terms()
            .map(|terms| self.store.read(OwnerSaid::before(terms, at.clone())))
            .transpose()?
            .unwrap_or_default();
        let grounding = Prompt::from(&Grounding::new(truly, end.replied(), &history));
        let truth = Truth::read(&self.crew.called(&grounding, Self::TIMEOUT, Model::Sonnet)?, &history);
        let selected = self.crew.called(&Prompt::from(&asked.selecting(&candidates)), Self::TIMEOUT, Model::Haiku)?;
        let stored = Arm::picked(&selected, &candidates);
        let whole: Vec<Unabridged<'_>> = history.iter().map(Unabridged::from).collect();
        let read = self.crew.called(&Prompt::from(&asked.selecting(&whole)), Self::TIMEOUT, Model::Haiku)?;
        let based = Arm::picked(&read, &history);
        let chunks = history.chunks(Self::CHUNK).map(|chunk| self.chunk_read(asked, chunk));
        let chunked = Arm::concat(chunks.collect::<Result<Vec<Arm>, Failure>>()?);
        let store_checked = self.crew.called(&Prompt::from(&stored.verifying(truly)), Self::TIMEOUT, Model::Sonnet)?;
        let base_checked = self.crew.called(&Prompt::from(&based.verifying(truly)), Self::TIMEOUT, Model::Sonnet)?;
        let chunk_checked = self.crew.called(&Prompt::from(&chunked.verifying(truly)), Self::TIMEOUT, Model::Sonnet)?;
        Ok(Judged::new(
            truth,
            &candidates,
            [stored.checked(&store_checked), based.checked(&base_checked), chunked.checked(&chunk_checked)],
        ))
    }

    fn chunk_read(&self, asked: &Asked, chunk: &[Said]) -> Result<Arm, Failure> {
        let whole: Vec<Unabridged<'_>> = chunk.iter().map(Unabridged::from).collect();
        let read = self.crew.called(&Prompt::from(&asked.selecting(&whole)), Self::TIMEOUT, Model::Haiku)?;
        Ok(Arm::picked(&read, chunk))
    }
}
