use graph::{Domain, Resolved};

use super::boundary::ScopeId;
use super::items::ItemIds;
use super::resolver::Resolver;
use super::use_path::UsePath;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Resolving<'ids, 'tables, 'rows> {
    resolver: Resolver<'tables, 'rows>,
    ids: &'ids ItemIds,
    base: ScopeId,
}

impl<'ids, 'tables, 'rows> Resolving<'ids, 'tables, 'rows> {
    pub(crate) fn new(resolver: Resolver<'tables, 'rows>, ids: &'ids ItemIds) -> Resolving<'ids, 'tables, 'rows> {
        Resolving { resolver, ids, base: ScopeId::FIRST }
    }

    pub(crate) fn based(self, base: ScopeId) -> Resolving<'ids, 'tables, 'rows> {
        Resolving { base, ..self }
    }

    pub(crate) fn path(&self, scope: ScopeId, path: &UsePath, domain: Domain) -> Resolved {
        self.resolver.path(path, scope.above(self.base), domain).forced().resolved(self.ids, path.written())
    }
}
