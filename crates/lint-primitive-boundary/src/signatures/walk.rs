use std::path::Path;

use code_model::{ItemPath, Named, Projection, Resolved, Sited, Spelling, TypeRef, TypeUses};
use lint_layout::LineNumber;

use crate::judged::Candidate;
use crate::legacy::Legacy;
use crate::lexicon::{Identity, Lexicon, Lone};
use crate::lookups::{AliasTarget, Lookups};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Walking {
    Direct,
    AliasTop,
    Nested,
}

impl Walking {
    fn inner(self) -> Walking {
        match self {
            Walking::Direct => Walking::Direct,
            Walking::AliasTop | Walking::Nested => Walking::Nested,
        }
    }

    fn aliased(self, legacy: Legacy) -> Walking {
        match self {
            Walking::Direct => Walking::of_alias(legacy),
            Walking::AliasTop | Walking::Nested => Walking::Nested,
        }
    }

    fn of_alias(legacy: Legacy) -> Walking {
        match legacy {
            Legacy::Eligible => Walking::AliasTop,
            Legacy::Ineligible => Walking::Nested,
        }
    }

    pub(super) fn legacy(self, identity: &Identity, written: &ItemPath) -> Legacy {
        match Lone::within(written) {
            Some(lone) => Spelt { identity, lone }.single(self),
            None => self.pathed(identity),
        }
    }

    fn pathed(self, identity: &Identity) -> Legacy {
        match self {
            Walking::Direct | Walking::AliasTop => identity.rerooted(),
            Walking::Nested => Legacy::Ineligible,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Spelt<'spelt> {
    identity: &'spelt Identity,
    lone: Lone<'spelt>,
}

impl Spelt<'_> {
    fn single(self, walking: Walking) -> Legacy {
        match walking {
            Walking::Direct => self.lone.primitive_or_rerooted(self.identity),
            Walking::AliasTop => self.lone.primitive(),
            Walking::Nested => Legacy::Ineligible,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Depth(u8);

impl Depth {
    const LIMIT: Depth = Depth(16);

    fn deeper(self) -> Option<Depth> {
        self.0.checked_sub(1).map(Depth)
    }
}

#[derive(Clone, Copy, Debug)]
struct Entering<'walk> {
    target: &'walk TypeRef,
    walking: Walking,
    line: Option<LineNumber>,
    depth: Depth,
}

#[derive(Debug)]
pub struct TypeWalk<'walk, 'model> {
    lexicon: &'walk Lexicon,
    lookups: &'walk Lookups<'model>,
    file: &'walk Path,
    walking: Walking,
    line: Option<LineNumber>,
    depth: Depth,
    found: Vec<Candidate>,
}

impl<'walk, 'model> TypeWalk<'walk, 'model> {
    pub fn walked<'ty>(
        lexicon: &'walk Lexicon,
        lookups: &'walk Lookups<'model>,
        file: &'walk Path,
        types: impl Iterator<Item = &'ty TypeRef>,
    ) -> Vec<Candidate> {
        let mut walk = TypeWalk {
            lexicon,
            lookups,
            file,
            walking: Walking::Direct,
            line: None,
            depth: Depth::LIMIT,
            found: Vec::new(),
        };
        let walked = types.fold(&mut walk, |walk, ty| {
            ty.each_use(walk);
            walk
        });
        std::mem::take(&mut walked.found)
    }

    fn spelled(&mut self, named: &Named<Sited>, written: &ItemPath) {
        let resolved: &Resolved = named.as_ref();
        let spelling: &Spelling = AsRef::<Sited>::as_ref(named).as_ref();
        let line = self.line.or(spelling.line());
        let identity = self.lexicon.identity(resolved, written);
        let walking = self.walking;
        let candidate = identity.zip(line).map(|(identity, line)| {
            let legacy = walking.legacy(&identity, written);
            Candidate::new(identity, line, legacy)
        });
        self.found.extend(candidate);
        let lookups = self.lookups;
        lookups.alias(resolved).into_iter().fold(&mut *self, |walk, alias| {
            walk.aliased(alias, written, line);
            walk
        });
        let outer = std::mem::replace(&mut self.walking, walking.inner());
        named.each_inner_use(self);
        self.walking = outer;
    }

    fn aliased(&mut self, alias: &AliasTarget<'model>, written: &ItemPath, line: Option<LineNumber>) {
        let walking = self.walking.aliased(alias.same_file(self.file).both(Lone::legacy(written)));
        let target = alias.target();
        let entering = self.depth.deeper().map(|depth| Entering { target, walking, line, depth });
        entering.into_iter().fold(self, |walk, entering| {
            walk.entered(entering);
            walk
        });
    }

    fn entered(&mut self, entering: Entering<'_>) {
        let walking = std::mem::replace(&mut self.walking, entering.walking);
        let line = std::mem::replace(&mut self.line, entering.line);
        let depth = std::mem::replace(&mut self.depth, entering.depth);
        entering.target.each_use(self);
        self.walking = walking;
        self.line = line;
        self.depth = depth;
    }
}

impl TypeUses for TypeWalk<'_, '_> {
    fn named(&mut self, named: &Named<Sited>) {
        AsRef::<Spelling>::as_ref(AsRef::<Sited>::as_ref(named)).written().into_iter().fold(self, |walk, written| {
            walk.spelled(named, written);
            walk
        });
    }

    fn projected(&mut self, projection: &Projection<Sited>) {
        projection.each_inner_use(self);
        let line: LineNumber = *projection.as_ref();
        let lookups = self.lookups;
        let resolved = lookups.projected(projection);
        let entering = self.depth.deeper().zip(resolved).map(|(depth, target)| Entering {
            target,
            walking: Walking::Nested,
            line: Some(line),
            depth,
        });
        entering.into_iter().fold(self, |walk, entering| {
            walk.entered(entering);
            walk
        });
    }
}
