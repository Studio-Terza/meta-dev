mod root;
mod scope;

use syn::{Item, Path};

use code_model::{Resolved, TraitPath};
use lint_layout::Name;

use root::Root;
pub(super) use scope::Reach;
use scope::{Meaning, Scope};

#[derive(Debug)]
pub(super) struct Scopes {
    open: Vec<Scope>,
}

impl Scopes {
    pub(super) fn new(items: &[Item]) -> Scopes {
        Scopes { open: vec![Scope::new(Reach::Module, items)] }
    }

    pub(super) fn enter<'item>(&mut self, reach: Reach, items: impl IntoIterator<Item = &'item Item>) {
        self.open.push(Scope::new(reach, items));
    }

    pub(super) fn leave(&mut self) {
        self.open.pop();
    }

    pub(super) fn resolved(&self, path: &Path) -> Resolved {
        match path.leading_colon {
            Some(_) => Root::rooted(Self::written(path).as_ref()),
            None => self.scoped(&Self::written(path)),
        }
    }

    pub(super) fn scoped(&self, path: &TraitPath) -> Resolved {
        self.meaning(path.as_ref()).resolved(path.as_ref())
    }

    fn written(path: &Path) -> TraitPath {
        path.segments.iter().map(|segment| Name::from(&segment.ident)).collect()
    }

    fn meaning(&self, segments: &[Name]) -> Meaning<'_> {
        let first = segments.first().filter(|first| Root::from(*first) == Root::Scoped);
        let start = self.open.iter().rposition(|scope| scope.reach() == Reach::Module).unwrap_or_default();
        let visible = self.open.get(start..).unwrap_or_default();
        let named = first.and_then(|first| visible.iter().rev().find_map(|scope| scope.meaning(first)));
        let only = segments.split_first().filter(|(_, rest)| rest.is_empty()).map(|(only, _)| only);
        let prelude = only.and_then(|only| Root::PRELUDE.iter().find(|[.., last]| *only == Name::from(*last)));
        named.or_else(|| prelude.map(|canonical| Meaning::Prelude(canonical))).unwrap_or(Meaning::Written)
    }
}
