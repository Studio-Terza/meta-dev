use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::boundary::OpName;
use super::op::{Op, Pass};
use super::run::Run;

pub trait Resource: Sized {
    const NAME: OpName;
    const RELEASE: OpName;
}

pub trait Release: Resource {
    fn release(&mut self, pass: Pass<'_, Released<Self>>) -> ReleaseReceipt;
}

#[derive(Debug)]
pub struct Released<R>(PhantomData<R>);

impl<R: Resource> Op for Released<R> {
    const NAME: OpName = R::RELEASE;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "a release receipt proves the resource was released; hand it back to the run"]
pub struct ReleaseReceipt(OpName);

impl<R: Resource> Pass<'_, Released<R>> {
    pub fn released(self) -> ReleaseReceipt {
        ReleaseReceipt(R::RELEASE)
    }
}

#[must_use = "a held resource is released when dropped"]
#[derive(Debug)]
pub struct Held<'run, R: Release> {
    resource: R,
    run: &'run Run,
}

impl<'run, R: Release> Held<'run, R> {
    pub(crate) fn acquired(resource: R, run: &'run Run) -> Self {
        Held { resource, run }
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

impl<R: Release> Drop for Held<'_, R> {
    fn drop(&mut self) {
        let ReleaseReceipt(released) = self.resource.release(Pass::minted());
        self.run.count(released);
    }
}
