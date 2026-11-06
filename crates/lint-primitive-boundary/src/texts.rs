use std::path::Path;

use code_model::{
    Call, Callee, Implementation, ItemPath, MacroCall, Model, PathCall, Placed, Reach, Region, Resolved, Seat,
    Spelling, TypeRef, Zone,
};
use lint_core::Violation;
use lint_layout::{Name, Whole, Word, words};

use crate::finding::Finding;
use crate::judged::Judged;
use crate::legacy::{Exempting, Legacy, Listing};
use crate::lexicon::{Lexicon, Lone};

#[derive(Debug)]
struct Door<'model> {
    path: &'model Path,
    subject: &'model Resolved,
    named: Option<Word<'static>>,
    structural: Option<()>,
    zone: Exempting<Zone>,
}

impl<'model> Door<'model> {
    fn opened(lexicon: &Lexicon, placed: Placed<'model, &'model Implementation>) -> Option<Door<'model>> {
        let implementation = placed.entity();
        let region: &Region = implementation.as_ref();
        let subject = implementation.subject_type().named().map(AsRef::<Resolved>::as_ref);
        let interface = implementation.interface();
        let written = interface.and_then(|interface| AsRef::<Spelling>::as_ref(interface).written());
        let named = written.and_then(ItemPath::last).and_then(|word| word.matches::<Whole>(&Lexicon::DOOR_WORDS));
        let structural = interface.and_then(|interface| lexicon.door(interface)).map(|_| ());
        named.map(|_| ()).or(structural).and(subject).map(|subject| Door {
            path: placed.site().path(),
            subject,
            named,
            structural,
            zone: Exempting::new(region.zone(), region.reach()),
        })
    }

    fn legacy(&self, path: &Path, subject: &Resolved) -> Option<&Door<'model>> {
        let place = [self.path].into_iter().find(|door| *door == path);
        let owner = [self.subject].into_iter().find(|door| *door == subject);
        let open = self.zone.covering(Reach::Shipped).into_iter().find(|zone| **zone == Zone::Open);
        place.and(owner).and(open).and(self.named).map(|_| self)
    }

    fn structural(&self, subject: &Resolved, reach: Reach) -> Option<&Door<'model>> {
        let owner = [self.subject].into_iter().find(|door| *door == subject);
        let covered = self.zone.covering(reach);
        owner.and(covered).and(self.structural).map(|()| self)
    }
}

#[derive(Debug)]
struct Sighting {
    handler: Name,
    legacy: Legacy,
}

impl Sighting {
    const TEXT_METHODS: [Word<'static>; 10] = words![
        "split_whitespace",
        "split_once",
        "trim",
        "trim_start",
        "trim_end",
        "chars",
        "char_indices",
        "split_at_checked",
        "to_string",
        "push_str",
    ];
    const FROM: Word<'static> = Word::new("from");
    const FORMAT: Word<'static> = Word::new("format");

    pub(crate) fn sighted(lexicon: &Lexicon, callee: &Callee) -> Option<Sighting> {
        let method = callee.method().and_then(Sighting::method);
        method
            .or_else(|| callee.path().and_then(|call| Sighting::converted(lexicon, call)))
            .or_else(|| callee.invoked().and_then(Sighting::formatted))
    }

    fn method(name: &Name) -> Option<Sighting> {
        name.matches::<Whole>(&Self::TEXT_METHODS).map(|_| Sighting { handler: name.clone(), legacy: Legacy::Eligible })
    }

    fn converted(lexicon: &Lexicon, call: &PathCall) -> Option<Sighting> {
        let written: &ItemPath = call.as_ref();
        let from = written.last().and_then(|last| last.matches::<Whole>(&[Self::FROM]).map(|_| last));
        let owned = lexicon.owned_type(call.as_ref());
        let legacy = lexicon.string_from(written);
        from.and_then(|from| Sighting::string_from(Sighting { handler: from.clone(), legacy }, owned))
    }

    fn string_from(sighting: Sighting, owned: Option<&TypeRef>) -> Option<Sighting> {
        match sighting.legacy {
            Legacy::Eligible => Some(sighting),
            Legacy::Ineligible => owned.map(|_| sighting),
        }
    }

    fn formatted(invoked: &MacroCall) -> Option<Sighting> {
        let written: &ItemPath = invoked.as_ref();
        let format = written.last().and_then(|last| last.matches::<Whole>(&[Self::FORMAT]).map(|_| last));
        let legacy = Legacy::of_position(invoked.position()).both(Lone::legacy(written));
        format.map(|format| Sighting { handler: format.clone(), legacy })
    }
}

#[derive(Debug)]
pub struct Texts<'read, 'model> {
    lexicon: &'read Lexicon,
    doors: Vec<Door<'model>>,
}

impl<'read, 'model> Texts<'read, 'model> {
    pub fn doored(lexicon: &'read Lexicon, model: &'model Model) -> Texts<'read, 'model> {
        Texts {
            lexicon,
            doors: model.every_implementation().filter_map(|placed| Door::opened(lexicon, placed)).collect(),
        }
    }

    pub fn judged(&self, model: &'model Model) -> Vec<Judged<'model>> {
        model
            .calls()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).zone() == Zone::Open)
            .filter_map(|placed| self.call_judged(placed))
            .collect()
    }

    fn call_judged(&self, placed: Placed<'model, &'model Call>) -> Option<Judged<'model>> {
        let call = placed.entity();
        let region: Region = *AsRef::<Region>::as_ref(call);
        let seat: &Seat = call.as_ref();
        let subject = seat.subject().and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref);
        let path = placed.site().path();
        let legacy_door = subject.and_then(|subject| self.doors.iter().find_map(|door| door.legacy(path, subject)));
        let door =
            subject.and_then(|subject| self.doors.iter().find_map(|door| door.structural(subject, region.reach())));
        let sighting = Sighting::sighted(self.lexicon, call.callee());
        sighting.and_then(|sighting| {
            let blocking = Legacy::of_reach(region.reach()).both(sighting.legacy).both(Legacy::unless(legacy_door));
            let finding = Finding::TextOutsideDoor { handler: sighting.handler };
            Texts::listed(blocking, door).map(|listing| Judged::new(listing, Violation::new(placed.site(), finding)))
        })
    }

    fn listed(blocking: Legacy, door: Option<&Door<'_>>) -> Option<Listing> {
        match blocking {
            Legacy::Eligible => Some(Listing::Blocking),
            Legacy::Ineligible => Texts::unexempted(door),
        }
    }

    fn unexempted(door: Option<&Door<'_>>) -> Option<Listing> {
        match door {
            Some(_) => None,
            None => Some(Listing::Widened),
        }
    }
}
