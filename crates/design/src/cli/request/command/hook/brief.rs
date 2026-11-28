mod boundary;

use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use invariants::Ignoring;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::super::ingest::Ingest;
use super::answers::{Answering, Moment};
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub(crate) use boundary::Brief;

impl Brief {
    pub(crate) fn told<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let passed = || Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PreToolUse))));
        self.body().map_or_else(passed, |body| {
            let store = ground.store();
            ground.prepared(crew.scratch())?;
            Ingest::read(store, &ground.transcripts())?;
            let answers = Answering::new(store, crew).searched(body, Moment::Briefing).ignoring(Ignored::Unanswered);
            answers.flatten().map_or_else(passed, |found| self.rewritten(body, &found))
        })
    }
}
