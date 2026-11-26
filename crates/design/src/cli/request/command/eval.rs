mod boundary;

use design_store::{OwnerBefore, Said, TurnEnd, Unabridged};
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
        let grounding = Prompt::from(&Grounding::new(truly, end.replied(), &history));
        let truth = Truth::read(&self.crew.called(&grounding, Self::TIMEOUT, Model::Sonnet)?, &history);
        let whole: Vec<Unabridged<'_>> = history.iter().map(Unabridged::from).collect();
        let read = self.crew.called(&Prompt::from(&asked.selecting(&whole)), Self::TIMEOUT, Model::Haiku)?;
        let based = Arm::picked(&read, &history);
        let reading = asked.reading(&history);
        let fanned = self.crew.fanned(&reading, Self::TIMEOUT)?;
        let chunked = Arm::concat(reading.chunks().iter().zip(&fanned).map(|(chunk, call)| Arm::picked(call, chunk)));
        let base_checked = self.crew.called(&Prompt::from(&based.verifying(truly)), Self::TIMEOUT, Model::Sonnet)?;
        let chunk_checked = self.crew.called(&Prompt::from(&chunked.verifying(truly)), Self::TIMEOUT, Model::Sonnet)?;
        Ok(Judged::new(truth, [based.checked(&base_checked), chunked.checked(&chunk_checked)]))
    }
}
