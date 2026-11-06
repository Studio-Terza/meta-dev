use std::cell::RefCell;
use std::rc::Rc;

use crate::{Entry, Op};

pub trait Resource {
    type Port;
    type Failure;

    fn release(&mut self) -> Result<(), Self::Failure>;
}

#[derive(Debug)]
pub struct ReleaseLog<F>(RefCell<Vec<F>>);

impl<F> Default for ReleaseLog<F> {
    fn default() -> ReleaseLog<F> {
        ReleaseLog(RefCell::new(Vec::new()))
    }
}

impl<F> ReleaseLog<F> {
    fn record(&self, failures: impl IntoIterator<Item = F>) {
        self.0.borrow_mut().extend(failures);
    }

    #[must_use]
    pub fn drain(&self) -> Vec<F> {
        self.0.borrow_mut().drain(..).collect()
    }
}

#[must_use = "a held resource is released when dropped"]
#[derive(Debug)]
pub struct Held<R: Resource> {
    resource: Option<R>,
    log: Rc<ReleaseLog<R::Failure>>,
}

impl<R: Resource> Held<R> {
    pub fn new<O: Op<Port = R::Port>>(_entry: Entry<O>, resource: R, log: &Rc<ReleaseLog<R::Failure>>) -> Held<R> {
        Held { resource: Some(resource), log: Rc::clone(log) }
    }

    pub fn end(mut self) -> Result<(), R::Failure> {
        self.resource.take().map_or(Ok(()), |mut resource| resource.release())
    }
}

impl<R: Resource> Drop for Held<R> {
    fn drop(&mut self) {
        let failures = self.resource.as_mut().map(Resource::release).and_then(Result::err);
        self.log.record(failures);
    }
}
