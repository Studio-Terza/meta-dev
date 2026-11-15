use std::sync::Arc;

use syntax::{Attribute, ItemImpl, ItemMod};

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

impl Lowering<'_> {
    pub(super) fn saved(&mut self) -> Descend {
        self.frames.push(Frame::from(&*self));
        Descend::Into
    }

    pub(super) fn restored(&mut self) {
        let frame = self.frames.pop().unwrap_or_else(|| Frame::from(&*self));
        self.here = frame.here;
        self.enclosure = frame.enclosure;
        self.generics = frame.generics;
        self.context = frame.context;
    }

    pub(super) fn slotted(&mut self, item: &ItemMod) -> Descend {
        self.lowered.slots.push(Slot::new(&self.here, self.context.region(), item));
        Descend::Over
    }
}
