mod head;
mod route;
mod trail;

use crate::module::Module;
use crate::scopes::Scopes;
use head::Head;
use lint_layout::{Name, UsePath};
use route::Route;
use trail::Trail;

pub(crate) struct Resolver<'scopes> {
    scopes: &'scopes Scopes<'scopes>,
}

impl<'scopes> Resolver<'scopes> {
    pub(crate) const fn over(scopes: &'scopes Scopes<'scopes>) -> Self {
        Resolver { scopes }
    }

    pub(crate) fn candidates(&self, module: &Module<'scopes>, path: UsePath) -> Vec<UsePath> {
        self.resolved(module, path, &Trail::default())
    }

    pub(crate) fn resolved(&self, module: &Module<'scopes>, path: UsePath, trail: &Trail<'_>) -> Vec<UsePath> {
        path.segments().first().map(Head::from).map_or_else(Vec::new, |head| head.resolved(self, module, path, trail))
    }

    pub(crate) fn routed(
        &self,
        module: &Module<'scopes>,
        head: &Name,
        path: &UsePath,
        trail: &Trail<'_>,
    ) -> Vec<UsePath> {
        Route::leading(self.scopes, module, head, trail)
            .iter()
            .flat_map(|route| route.follow(self, module, path, trail))
            .collect()
    }
}
