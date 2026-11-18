use std::collections::HashMap;

use syntax::{Ident, Macro};

use graph::{Building, Domain, FileId, ItemId, ItemPath, Member, Mention, PathUse, Resolved, Seat};
use lint_core::Macros;
use lint_layout::LineNumber;

use super::boundary::ScopeId;
use super::items::ItemIds;
use super::resolver::Resolver;
use super::use_path::UsePath;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Resolving<'ids, 'tables, 'rows> {
    resolver: Resolver<'tables, 'rows>,
    ids: &'ids ItemIds,
    transcriptions: &'ids Transcriptions,
    base: ScopeId,
}

impl<'ids, 'tables, 'rows> Resolving<'ids, 'tables, 'rows> {
    pub(crate) fn new(
        resolver: Resolver<'tables, 'rows>,
        ids: &'ids ItemIds,
        transcriptions: &'ids Transcriptions,
    ) -> Resolving<'ids, 'tables, 'rows> {
        Resolving { resolver, ids, transcriptions, base: ScopeId::FIRST }
    }

    pub(crate) fn based(self, base: ScopeId) -> Resolving<'ids, 'tables, 'rows> {
        Resolving { base, ..self }
    }

    pub(crate) fn transcriptions(&self) -> &'ids Transcriptions {
        self.transcriptions
    }

    pub(crate) fn path(&self, scope: ScopeId, path: &UsePath, domain: Domain) -> Resolved {
        self.resolver.path(path, scope.above(self.base), domain).forced().resolved(self.ids, path.written())
    }
}

#[derive(Debug)]
pub(crate) struct TranscribedRow {
    scope: ScopeId,
    name: UsePath,
    mentioned: Vec<UsePath>,
}

#[derive(Debug)]
pub(crate) struct Transcription {
    defined: ItemId,
    transcribed: Vec<Transcribed>,
}

#[derive(Debug, Default)]
pub(crate) struct Transcriptions(HashMap<ItemId, Vec<Transcribed>>);

#[derive(Clone, Debug)]
enum Transcribed {
    Fixed(ItemPath, Resolved),
    Open(UsePath),
}

#[derive(Clone, Copy, Debug)]
struct Site<'grown> {
    resolving: Resolving<'grown, 'grown, 'grown>,
    scope: ScopeId,
}

#[derive(Clone, Debug)]
pub(crate) struct Invoked {
    file: FileId,
    line: LineNumber,
    seat: Seat,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Invoker<'grown, 'invoked> {
    site: Site<'grown>,
    invoked: &'invoked Invoked,
}

impl TranscribedRow {
    pub(crate) fn read(scope: ScopeId, ident: &Ident, definition: &Macro) -> TranscribedRow {
        let mentioned = Macros::mentioned(definition).paths().iter().map(UsePath::from).collect();
        TranscribedRow { scope, name: UsePath::from(ident), mentioned }
    }

    pub(crate) fn settled(&self, resolving: Resolving<'_, '_, '_>) -> Option<Transcription> {
        let defined = resolving.path(self.scope, &self.name, Domain::Macros).crate_item()?;
        let site = Site { resolving, scope: self.scope };
        let transcribed = self.mentioned.iter().map(|path| Transcribed::read(path, site)).collect();
        Some(Transcription { defined, transcribed })
    }
}

impl Invoked {
    pub(crate) fn new(file: FileId, line: LineNumber, seat: Seat) -> Invoked {
        Invoked { file, line, seat }
    }

    fn used(&self, written: ItemPath, resolved: Resolved) -> PathUse {
        PathUse::new(written, resolved, Mention::Expanded, self.seat.clone())
    }
}

impl<'grown, 'invoked> Invoker<'grown, 'invoked> {
    pub(crate) fn new(
        resolving: Resolving<'grown, 'grown, 'grown>,
        scope: ScopeId,
        invoked: &'invoked Invoked,
    ) -> Invoker<'grown, 'invoked> {
        Invoker { site: Site { resolving, scope }, invoked }
    }

    fn used(self, path: &UsePath) -> Option<PathUse> {
        self.mentioned(path.written(), self.site.found(path))
    }

    fn mentioned(self, written: ItemPath, resolved: Resolved) -> Option<PathUse> {
        resolved.known()?;
        Some(self.invoked.used(written, resolved))
    }
}

impl Transcriptions {
    pub(crate) fn settled(transcriptions: impl IntoIterator<Item = Transcription>) -> Transcriptions {
        Transcriptions(transcriptions.into_iter().map(|found| (found.defined, found.transcribed)).collect())
    }

    pub(crate) fn expanded(&self, definition: &Resolved, invoker: Invoker<'_, '_>, building: Building) -> Building {
        let transcribed = definition.crate_item().and_then(|item| self.0.get(&item)).map_or(&[][..], Vec::as_slice);
        transcribed.iter().filter_map(|path| path.used(invoker)).fold(building, |mut built, used| {
            built.path_use(invoker.invoked.file, invoker.invoked.line, used);
            built
        })
    }
}

impl Site<'_> {
    fn found(self, path: &UsePath) -> Resolved {
        let typed = self.resolving.path(self.scope, path, Domain::Types);
        let valued =
            || Some(self.resolving.path(self.scope, path, Domain::Values)).filter(|found| found.known().is_some());
        let member = || {
            let owner = self.resolving.path(self.scope, &path.qualifier()?, Domain::Types).crate_item()?;
            Some(Resolved::Member(Member::new(owner, path.tail().last()?.clone())))
        };
        typed.known().cloned().or_else(valued).or_else(member).unwrap_or(typed)
    }
}

impl Transcribed {
    fn read(path: &UsePath, site: Site<'_>) -> Transcribed {
        match path.crate_anchored() {
            Some(()) => Transcribed::Fixed(path.written(), site.found(path)),
            None => Transcribed::Open(path.clone()),
        }
    }

    fn used(&self, invoker: Invoker<'_, '_>) -> Option<PathUse> {
        match self {
            Transcribed::Fixed(written, resolved) => invoker.mentioned(written.clone(), resolved.clone()),
            Transcribed::Open(path) => invoker.used(path),
        }
    }
}
