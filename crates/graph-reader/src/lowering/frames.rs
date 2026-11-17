use std::sync::Arc;

use syntax::{Attribute, Block, ItemImpl, ItemMod};

use lint_sweep::{Descend, Visit};

use crate::namespace::{Entries, ScopeKind};

use super::regions::Context;
use super::seat::Seat;
use super::signature::InScope;
use super::{Lowering, Slot};
use graph::{Entry, Owner};
use lint_layout::{Name, TestMarks, Whole, Word, words};

#[derive(Clone, Copy, Debug, Default)]
pub(super) enum Enclosure {
    BinaryRoot,
    #[default]
    Module,
    Body,
    Impl(Owner),
    Trait,
    Foreign,
}

impl Enclosure {
    const MAIN: [Word<'static>; 1] = words!["main"];

    pub(super) fn of_impl(block: &ItemImpl) -> Enclosure {
        match block.trait_ {
            Some(_) => Enclosure::Impl(Owner::TraitImpl),
            None => Enclosure::Impl(Owner::Inherent),
        }
    }

    pub(super) fn owner(self) -> Owner {
        match self {
            Enclosure::BinaryRoot | Enclosure::Module => Owner::Free,
            Enclosure::Body => Owner::Nested,
            Enclosure::Impl(owner) => owner,
            Enclosure::Trait => Owner::TraitDecl,
            Enclosure::Foreign => Owner::Foreign,
        }
    }

    pub(super) fn entry(self, attributes: &[Attribute], name: &Name) -> Entry {
        TestMarks::mark_test(attributes).map(|_| Entry::Test).or_else(|| self.main(name)).unwrap_or(Entry::Called)
    }

    fn main(self, name: &Name) -> Option<Entry> {
        match self {
            Enclosure::BinaryRoot => name.matches::<Whole>(&Self::MAIN).map(|_| Entry::Main),
            Enclosure::Module | Enclosure::Body | Enclosure::Impl(_) | Enclosure::Trait | Enclosure::Foreign => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct Frame {
    here: Seat,
    enclosure: Enclosure,
    generics: Arc<InScope>,
    context: Context,
}

impl From<&Lowering<'_>> for Frame {
    fn from(lowering: &Lowering<'_>) -> Frame {
        Frame {
            here: lowering.here.clone(),
            enclosure: lowering.enclosure,
            generics: Arc::clone(&lowering.generics),
            context: lowering.context,
        }
    }
}

impl Frame {
    pub(super) fn saved(mut lowering: Box<Lowering<'_>>) -> Box<Lowering<'_>> {
        let frame = Frame::from(&*lowering);
        lowering.frames.push(frame);
        lowering
    }
}

impl<'read> Lowering<'read> {
    pub(super) fn enclosing(mut self: Box<Self>, enclosure: Enclosure) -> Box<Lowering<'read>> {
        self.enclosure = enclosure;
        self
    }

    pub(super) fn contexted(mut self: Box<Self>, change: impl FnOnce(Context) -> Context) -> Box<Lowering<'read>> {
        self.context = change(self.context);
        self
    }

    pub(super) fn enclosed(mut self: Box<Self>, enclosure: Enclosure, generics: InScope) -> Box<Lowering<'read>> {
        self.enclosure = enclosure;
        self.generics = Arc::new(generics);
        self
    }

    pub(super) fn restored(mut self: Box<Self>) -> Box<Lowering<'read>> {
        let frame = self.frames.pop().unwrap_or_else(|| Frame::from(&*self));
        self.here = frame.here;
        self.enclosure = frame.enclosure;
        self.generics = frame.generics;
        self.context = frame.context;
        self
    }

    pub(super) fn slotted(mut self: Box<Self>, item: &ItemMod) -> Visit<Box<Lowering<'read>>> {
        let slot = Slot::new(&self.here, self.context.region(), item);
        self.lowered.slots.push(slot);
        Visit::new(self, Descend::Over)
    }

    pub(super) fn block_entered(self: Box<Self>, block: &Block) -> Box<Lowering<'read>> {
        let children = self.here.children();
        Frame::saved(self).entered(ScopeKind::Block, Entries::of_block(&block.stmts), children)
    }
}
