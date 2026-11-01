mod route;
mod trail;

use crate::module::Module;
use crate::scopes::Scopes;
use lint_layout::UsePath;
use route::Route;
use trail::Trail;

pub(crate) struct Resolver<'scopes> {
    scopes: &'scopes Scopes,
}

impl<'scopes> Resolver<'scopes> {
    const CRATE: &'static str = "crate";
    const SELF: &'static str = "self";
    const SUPER: &'static str = "super";

    pub(crate) fn over(scopes: &'scopes Scopes) -> Resolver<'scopes> {
        Resolver { scopes }
    }

    pub(crate) fn candidates(&self, module: &Module, path: &UsePath) -> Vec<UsePath> {
        self.resolved(module, path, &Trail::default())
    }

    fn resolved(&self, module: &Module, path: &UsePath, trail: &Trail) -> Vec<UsePath> {
        let rest = path.rest();
        match path.first() {
            Some(head) if *head == Self::CRATE => self.resolved(&module.root(), &rest, trail),
            Some(head) if *head == Self::SELF => self.resolved(module, &rest, trail),
            Some(head) if *head == Self::SUPER => {
                module.parent().map_or_else(Vec::new, |parent| self.resolved(&parent, &rest, trail))
            }
            Some(head) => Route::leading(self.scopes, module, head, trail)
                .iter()
                .flat_map(|route| route.follow(self, module, path, trail))
                .chain(Some(path.clone()))
                .collect(),
            None => Vec::new(),
        }
    }
}
