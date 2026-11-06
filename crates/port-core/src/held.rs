use std::marker::PhantomData;

use super::op::{Op, OpName, Pass};
use super::run::{ReleaseFailure, Run};

pub trait Resource: Sized {
    const NAME: OpName;
    const RELEASE: OpName;
}

pub trait Release: Resource {
    type Failure: std::error::Error + Send + Sync + 'static;

    fn release(&mut self, pass: Pass<'_, Released<Self>>) -> Result<(), Self::Failure>;
}

#[derive(Debug)]
pub struct Released<R>(PhantomData<R>);

impl<R: Resource> Op for Released<R> {
    const NAME: OpName = R::RELEASE;
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

    #[must_use]
    pub fn resource(&self) -> Option<&R> {
        self.resource.as_ref()
    }

    pub fn end(mut self) -> Result<(), R::Failure> {
        let run = self.run;
        self.resource.take().map_or(Ok(()), |mut resource| {
            run.count(R::RELEASE);
            resource.release(Pass::minted())
        })
    }
}

impl<R: Release> Drop for Held<'_, R> {
    fn drop(&mut self) {
        let run = self.run;
        self.resource.take().into_iter().for_each(|resource| {
            Held { resource: Some(resource), run }
                .end()
                .err()
                .into_iter()
                .for_each(|failure| run.fail(ReleaseFailure::new(R::NAME, Box::new(failure))));
        });
    }
}
