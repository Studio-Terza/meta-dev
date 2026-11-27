use std::path::Path;

use design_messages::Tally;
use invariants::Ignoring;
use newtype::Wrapping;
use ports::dispatch::{
    Bell, Call, Clock, Deadline, Dispatch, Finished, Job, Model, Moment, Pause, Prompt, Ran, Reply, Shell, Stall,
    Timeout, Usage, Wall, Watch, Watching,
};

use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;

#[derive(Debug)]
pub struct Crew<'ports, D: Dispatch, C: Clock, G: Shell> {
    dispatch: &'ports D,
    clock: &'ports C,
    shell: &'ports G,
}

#[derive(Clone, Copy, Debug)]
struct Replying;

#[derive(Clone, Debug, Default)]
pub struct Heard {
    reply: Option<Reply>,
    tokens: Tally,
}

impl<'ports, D: Dispatch, C: Clock, G: Shell> Crew<'ports, D, C, G> {
    pub fn new(dispatch: &'ports D, clock: &'ports C, shell: &'ports G) -> Crew<'ports, D, C, G> {
        Crew { dispatch, clock, shell }
    }

    pub fn now(&self) -> Wall {
        self.clock.now()
    }

    pub fn spoke(&self, program: &'static str, arguments: &[&str], directory: &Path, timeout: Timeout) -> Option<Ran> {
        let spoken = arguments.iter().map(|word| (*word).to_owned()).collect();
        let deadline = Deadline::after(self.clock.now(), timeout);
        let call = Call::new(program, spoken, directory.to_path_buf(), deadline);
        self.shell.ran(&call).ignoring(Ignored::Unrun)
    }

    pub fn moment(&self) -> Moment {
        self.clock.moment()
    }

    pub fn paused(&self, pause: Pause) {
        self.clock.waited(pause, &Bell::hung());
    }

    pub fn replied(&self, prompt: &Prompt, timeout: Timeout, model: Model) -> Result<Heard, Failure> {
        let held = self.dispatch.dispatch(&Job::due_by(Deadline::after(self.clock.now(), timeout), model), prompt)?;
        Ok(Watch::over(self.dispatch, self.clock).watched(held)?.visited(Replying))
    }

    pub fn fanned(&self, prompts: &[Prompt], timeout: Timeout) -> Result<Vec<Heard>, Failure> {
        let job = Job::due_by(Deadline::after(self.clock.now(), timeout), Model::Haiku);
        let watched = Watch::over(self.dispatch, self.clock).fanned(&job, prompts)?;
        Ok(watched.iter().map(|each| each.visited(Replying)).collect())
    }
}

impl Heard {
    fn spent(usage: Option<Usage>) -> Tally {
        usage.map_or_default(|used| {
            used.spent().iter().flat_map(|tokens| usize::try_from(tokens.into_inner())).map(Tally::from).sum()
        })
    }

    pub fn reply(&self) -> Option<&Reply> {
        self.reply.as_ref()
    }

    pub fn tokens(&self) -> Tally {
        self.tokens
    }
}

impl Watching for Replying {
    type Output = Heard;

    fn finished(self, finished: &Finished) -> Heard {
        Heard { reply: Some(finished.reply().clone()), tokens: Heard::spent(Some(finished.usage())) }
    }

    fn stalled(self, stall: &Stall) -> Heard {
        Heard { reply: None, tokens: Heard::spent(stall.usage()) }
    }
}
