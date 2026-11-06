use dispatch_ports::{Clock, Dispatch};

#[derive(Debug)]
pub struct Crew<'run, D: Dispatch, C: Clock> {
    dispatch: &'run D,
    clock: &'run C,
}

impl<'run, D: Dispatch, C: Clock> Crew<'run, D, C> {
    pub fn new(dispatch: &'run D, clock: &'run C) -> Crew<'run, D, C> {
        Crew { dispatch, clock }
    }

    pub fn dispatch(&self) -> &'run D {
        self.dispatch
    }

    pub fn clock(&self) -> &'run C {
        self.clock
    }
}
