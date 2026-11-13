use std::sync::Arc;

use syn::{Attribute, ItemImpl, ItemMod};

use lint_sweep::Descend;

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

impl Frame {
    pub(super) fn enclosure(&self) -> Enclosure {
        self.enclosure
    }
}

impl Lowering {
    pub(super) fn frame(&self) -> Frame {
        Frame {
            here: self.here.clone(),
            enclosure: self.enclosure,
            generics: Arc::clone(&self.generics),
            context: self.context,
        }
    }

    pub(super) fn saved(&mut self, frame: Frame) -> Descend {
        self.frames.push(frame);
        Descend::Into
    }

    pub(super) fn restored(&mut self) {
        let frame = self.frames.pop().unwrap_or_else(|| self.frame());
        self.here = frame.here;
        self.enclosure = frame.enclosure;
        self.generics = frame.generics;
        self.context = frame.context;
    }

    pub(super) fn slotted(&mut self, item: &ItemMod) -> Descend {
        self.lowered.slots.push(Slot::new(&self.here, self.context.region(), item));
        self.frames.push(self.frame());
        Descend::Over
    }
}
