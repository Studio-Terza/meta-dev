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
    const SPELLED: [(&'static str, Self); 3] =
        [("crate", Self::Crate), ("self", Self::Current), ("super", Self::Super)];

    pub(crate) fn resolved<'scopes>(
        self,
        resolver: &Resolver<'scopes>,
        module: &Module<'scopes>,
        path: UsePath,
        trail: &Trail<'_>,
    ) -> Vec<UsePath> {
        match self {
            Self::Crate => resolver.resolved(&Module::from(<&CrateFile>::from(module)), path.rest(), trail),
            Self::Current => resolver.resolved(module, path.rest(), trail),
            Self::Super => {
                module.parent().map_or_else(Vec::new, |parent| resolver.resolved(&parent, path.rest(), trail))
            }
            Self::Named => path
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
    fn from(name: &Name) -> Self {
        Self::SPELLED
            .into_iter()
            .find(|(spelled, _)| *name == Name::from(*spelled))
            .map_or(Self::Named, |(_, head)| head)
    }
}
