use std::error::Error;
use std::marker::PhantomData;

use super::op::{Op, OpName, Pass};
use super::run::Run;

pub trait Resource: Sized {
    const NAME: OpName;
    const RELEASE: OpName;
}

pub trait Release: Resource {
    type Failure: Error + Send + Sync + 'static;

    fn release(&mut self, pass: Pass<'_, Released<Self>>) -> Result<ReleaseReceipt, Self::Failure>;
}

#[derive(Debug)]
pub struct Released<R>(PhantomData<R>);

impl<R: Resource> Op for Released<R> {
    const NAME: OpName = R::RELEASE;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseReceipt(OpName);

impl<R: Resource> Pass<'_, Released<R>> {
    #[must_use]
    pub fn released(self) -> ReleaseReceipt {
        ReleaseReceipt(R::RELEASE)
    }
}

#[must_use = "a held resource is released when dropped"]
#[derive(Debug)]
pub struct Held<'run, R: Release> {
    resource: Option<R>,
    run: &'run Run,
}

impl<'run, R: Release> Held<'run, R> {
    pub(crate) fn acquired(resource: R, run: &'run Run) -> Self {
        Held { resource: Some(resource), run }
    }

    pub fn end(mut self) -> Result<ReleaseReceipt, R::Failure> {
        let run = self.run;
        self.resource.take().map_or(Ok(ReleaseReceipt(R::RELEASE)), |resource| run.released(resource))
    }
}

impl Run {
    fn released<R: Release>(&self, mut resource: R) -> Result<ReleaseReceipt, R::Failure> {
        self.count(R::RELEASE);
        resource.release(Pass::minted())
    }
}

impl<R: Release> Drop for Held<'_, R> {
    fn drop(&mut self) {
        let run = self.run;
        self.resource
            .take()
            .and_then(|resource| run.released(resource).err())
            .into_iter()
            .for_each(|failure| run.fail(R::NAME, Box::new(failure)));
    }
}
