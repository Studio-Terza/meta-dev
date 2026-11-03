use super::Resolver;
use super::trail::{Globbing, Trail};
use crate::module::Module;
use crate::scopes::Scopes;
use lint_layout::{Name, UsePath};

#[derive(Debug)]
pub(crate) enum Route<'scope> {
    Imported { head: &'scope Name, target: &'scope UsePath },
    Entered(Module<'scope>),
    Globbed(&'scope UsePath),
}

impl<'scope> Route<'scope> {
    pub(crate) fn leading(
        scopes: &'scope Scopes<'scope>,
        module: &Module<'scope>,
        head: &'scope Name,
        trail: &Trail<'_>,
    ) -> Vec<Route<'scope>> {
        let scope = scopes.scope(module);
        let imported = scope
            .filter(|_| trail.followed(module, head).is_none())
            .and_then(|scope| scope.imported(head))
            .map(|target| Route::Imported { head, target });
        let entered =
            Some(module.within(head.clone())).filter(|child| scopes.scope(child).is_some()).map(Route::Entered);
        let globbed = scope
            .filter(|_| imported.is_none())
            .filter(|_| trail.globbing() == Globbing::Open)
            .into_iter()
            .flat_map(|scope| scope.globs().iter());
        imported.into_iter().chain(entered).chain(globbed.map(Route::Globbed)).collect()
    }

    pub(crate) fn follow(
        &self,
        resolver: &Resolver<'scope>,
        module: &Module<'scope>,
        path: &UsePath,
        trail: &Trail<'_>,
    ) -> Vec<UsePath> {
        match self {
            Route::Imported { head, target } => {
                resolver.resolved(module, path.rerooted(target), &trail.following(module, head))
            }
            Route::Entered(child) => {
                path.segments().get(1).map_or_else(Vec::new, |_| resolver.resolved(child, path.rest(), trail))
            }
            Route::Globbed(glob) => resolver.resolved(module, glob.joined(path.segments()), &trail.spent()),
        }
    }
}
