mod route;
mod trail;

use crate::module::Module;
use crate::scopes::Scopes;
use lint_layout::UsePath;
use lint_placement::CrateFile;
use route::Route;
use trail::Trail;

pub(crate) struct Resolver<'scopes> {
    scopes: &'scopes Scopes<'scopes>,
}

impl<'scopes> Resolver<'scopes> {
    const CRATE: &'static str = "crate";
    const SELF: &'static str = "self";
    const SUPER: &'static str = "super";

    pub(crate) fn over(scopes: &'scopes Scopes<'scopes>) -> Resolver<'scopes> {
        Resolver { scopes }
    }

    pub(crate) fn candidates(&self, module: &Module<'scopes>, path: UsePath) -> Vec<UsePath> {
        self.resolved(module, path, &Trail::default())
    }

    fn resolved(&self, module: &Module<'scopes>, path: UsePath, trail: &Trail<'_>) -> Vec<UsePath> {
        match path.segments().first() {
            Some(head) if *head == Self::CRATE => {
                self.resolved(&Module::from(<&CrateFile>::from(module)), path.rest(), trail)
            }
            Some(head) if *head == Self::SELF => self.resolved(module, path.rest(), trail),
            Some(head) if *head == Self::SUPER => {
                module.parent().map_or_else(Vec::new, |parent| self.resolved(&parent, path.rest(), trail))
            }
            Some(head) => {
                let mut found: Vec<UsePath> = Route::leading(self.scopes, module, head, trail)
                    .iter()
                    .flat_map(|route| route.follow(self, module, &path, trail))
                    .collect();
                found.push(path);
                found
            }
            None => Vec::new(),
        }
    }
}
