use code_model::Resolved;

use super::boundary::ScopeId;
use super::items::ItemIds;
use super::resolver::Resolver;
use super::tables::Domain;
use super::use_path::UsePath;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Resolving<'ids, 'tables, 'rows> {
    resolver: Resolver<'tables, 'rows>,
    ids: &'ids ItemIds,
}

impl<'ids, 'tables, 'rows> Resolving<'ids, 'tables, 'rows> {
    pub(crate) fn new(resolver: Resolver<'tables, 'rows>, ids: &'ids ItemIds) -> Resolving<'ids, 'tables, 'rows> {
        Resolving { resolver, ids }
    }

    pub(crate) fn path(&self, scope: ScopeId, path: &UsePath, domain: Domain) -> Resolved {
        path.bound(domain, &self.resolver, scope).forced().resolved(self.ids, path.written())
    }
}
