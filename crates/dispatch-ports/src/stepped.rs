use std::sync::{Arc, Mutex};
use std::time::Duration;

use invariants::Unpoisoned;
use port_core::Pass;
use serde::Deserialize;

use crate::{Bell, ClockFiller, Count, Moment, Pause, Wall, Woke, clock_ops};

#[derive(Clone, Debug)]
pub struct Stepped {
    start: Wall,
    oversleep: Option<Oversleep>,
    walked: Arc<Mutex<Walked>>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Oversleep {
    after: Count,
    by: Pause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wait {
    pause: Pause,
    woke: Woke,
}

#[derive(Debug, Default)]
struct Walked {
    now: Option<Wall>,
    waits: Vec<Wait>,
}

impl Stepped {
    const STILL: Pause = Pause::new(0);

    #[must_use]
    pub fn starting(start: Wall, oversleep: Option<Oversleep>) -> Stepped {
        Stepped { start, oversleep, walked: Arc::default() }
    }

    #[must_use]
    pub fn waits(&self) -> Vec<Wait> {
        self.walked.lock().unpoisoned().waits.clone()
    }

    fn current(&self, walked: &Walked) -> Wall {
        walked.now.unwrap_or(self.start)
    }
}

impl Oversleep {
    #[must_use]
    pub fn new(after: Count, by: Pause) -> Oversleep {
        Oversleep { after, by }
    }
}

impl Wait {
    fn granted(self) -> Pause {
        match self.woke {
            Woke::Lapsed => self.pause,
            Woke::Rung => Stepped::STILL,
        }
    }
}

impl ClockFiller for Stepped {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        self.current(&self.walked.lock().unpoisoned())
    }

    fn moment(&self, _pass: Pass<'_, clock_ops::Moment>) -> Moment {
        Moment::from(self.current(&self.walked.lock().unpoisoned()))
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, bell: &Bell) -> Wall {
        let wait = Wait { pause, woke: bell.waited(Duration::ZERO) };
        let mut walked = self.walked.lock().unpoisoned();
        let slept = self.oversleep.filter(|oversleep| oversleep.after.reached(walked.waits.len()).is_some());
        let now =
            self.current(&walked).after(wait.granted()).after(slept.map_or(Stepped::STILL, |oversleep| oversleep.by));
        walked.stepped(wait, now);
        now
    }
}

impl Walked {
    fn stepped(&mut self, wait: Wait, now: Wall) {
        self.waits.push(wait);
        self.now = Some(now);
    }
}
