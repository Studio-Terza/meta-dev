use std::error::Error;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Holding {
    Held,
    Released,
}

#[must_use = "a held resource is released when dropped"]
#[derive(Debug)]
pub struct Held<'run, R: Release> {
    resource: R,
    holding: Holding,
    run: &'run Run,
}

impl<'run, R: Release> Held<'run, R> {
    pub(crate) fn acquired(resource: R, run: &'run Run) -> Self {
        Held { resource, holding: Holding::Held, run }
    }

    pub fn end(mut self) -> Result<ReleaseReceipt, R::Failure> {
        match std::mem::replace(&mut self.holding, Holding::Released) {
            Holding::Held => self.run.released(&mut self.resource),
            Holding::Released => Ok(ReleaseReceipt(R::RELEASE)),
        }
    }
}

impl<R: Release> Deref for Held<'_, R> {
    type Target = R;

    fn deref(&self) -> &R {
        &self.resource
    }
}

impl<R: Release> DerefMut for Held<'_, R> {
    fn deref_mut(&mut self) -> &mut R {
        &mut self.resource
    }
}

impl Run {
    fn released<R: Release>(&self, resource: &mut R) -> Result<ReleaseReceipt, R::Failure> {
        self.count(R::RELEASE);
        resource.release(Pass::minted())
    }

    fn dropped<R: Release>(&self, resource: &mut R) {
        self.released(resource).err().into_iter().for_each(|failure| self.fail(R::NAME, Box::new(failure)));
    }
}

impl<R: Release> Drop for Held<'_, R> {
    fn drop(&mut self) {
        match std::mem::replace(&mut self.holding, Holding::Released) {
            Holding::Held => self.run.dropped(&mut self.resource),
            Holding::Released => (),
        }
    }
}
