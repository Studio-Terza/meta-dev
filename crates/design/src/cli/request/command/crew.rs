mod boundary;

use design_messages::Tally;
use dispatch_ports::{
    Bell, Clock, Deadline, Dispatch, Finished, Job, Model, Moment, Pause, Prompt, Reply, Shell, Stall, Timeout, Usage,
    Wall, Watch, Watching,
};
use newtype::Wrapping;

use crate::cli::failure::Failure;

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

    pub(crate) fn moment(&self) -> Moment {
        self.clock.moment()
    }

    pub(crate) fn paused(&self, pause: Pause) {
        self.clock.waited(pause, &Bell::hung());
    }

    pub(crate) fn replied(&self, prompt: &Prompt, timeout: Timeout, model: Model) -> Result<Heard, Failure> {
        let held = self.dispatch.dispatch(&Job::due_by(Deadline::after(self.clock.now(), timeout), model), prompt)?;
        Ok(Watch::over(self.dispatch, self.clock).watched(held)?.visited(Replying))
    }

    pub(crate) fn fanned(&self, prompts: &[Prompt], timeout: Timeout) -> Result<Vec<Heard>, Failure> {
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
