use super::Resolver;
use super::trail::{Globbing, Trail};
use crate::module::Module;
use crate::scopes::Scopes;
use lint_layout::{Name, UsePath};

#[derive(Debug)]
pub(crate) enum Route {
    Imported { head: Name, target: UsePath },
    Entered(Module),
    Globbed(UsePath),
}

impl Route {
    pub(crate) fn leading(scopes: &Scopes, module: &Module, head: &Name, trail: &Trail) -> Vec<Route> {
        let scope = scopes.scope(module);
        let imported = scope
            .filter(|_| !trail.has_followed(module, head))
            .and_then(|scope| scope.imported(head))
            .map(|target| Route::Imported { head: head.clone(), target: target.clone() });
        let entered = Some(module.within(head)).filter(|child| scopes.declares(child)).map(Route::Entered);
        let globbed = scope
            .filter(|_| imported.is_none() && trail.globbing() == Globbing::Open)
            .into_iter()
            .flat_map(|scope| scope.globs().iter());
        imported
            .into_iter()
            .chain(entered)
            .chain(globbed.map(|glob| Route::Globbed(glob.extended(head.clone()))))
            .collect()
    }

    pub(crate) fn follow(&self, resolver: &Resolver, module: &Module, path: &UsePath, trail: &Trail) -> Vec<UsePath> {
        match self {
            Route::Imported { head, target } => {
                resolver.resolved(module, &path.rerooted(target), &trail.following(module, head))
            }
            Route::Entered(child) => Some(path.rest())
                .filter(|rest| !rest.segments().is_empty())
                .map_or_else(Vec::new, |rest| resolver.resolved(child, &rest, trail)),
            Route::Globbed(prefix) => resolver.resolved(module, &path.rerooted(prefix), &trail.spent()),
        }
    }
}
