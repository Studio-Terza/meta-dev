use std::iter;

use super::Resolver;
use super::trail::Trail;
use crate::module::Module;
use lint_layout::{Name, UsePath};
use lint_placement::CrateFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Head {
    Crate,
    Current,
    Super,
    Named,
}

impl Head {
    const SPELLED: [(&'static str, Head); 3] =
        [("crate", Head::Crate), ("self", Head::Current), ("super", Head::Super)];

    pub(crate) fn resolved<'scopes>(
        self,
        resolver: &Resolver<'scopes>,
        module: &Module<'scopes>,
        path: UsePath,
        trail: &Trail<'_>,
    ) -> Vec<UsePath> {
        match self {
            Head::Crate => resolver.resolved(&Module::from(<&CrateFile>::from(module)), path.rest(), trail),
            Head::Current => resolver.resolved(module, path.rest(), trail),
            Head::Super => {
                module.parent().map_or_else(Vec::new, |parent| resolver.resolved(&parent, path.rest(), trail))
            }
            Head::Named => path
                .segments()
                .first()
                .map(|head| resolver.routed(module, head, &path, trail))
                .unwrap_or_default()
                .into_iter()
                .chain(iter::once(path))
                .collect(),
        }
    }
}

impl From<&Name> for Head {
    fn from(name: &Name) -> Head {
        Head::SPELLED
            .into_iter()
            .find(|(spelled, _)| *name == Name::from(*spelled))
            .map_or(Head::Named, |(_, head)| head)
    }
}
