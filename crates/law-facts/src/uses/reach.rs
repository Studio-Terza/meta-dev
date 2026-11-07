use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Reach {
    Home,
    Crate(CrateName),
}

impl Reach {
    const HOME_ROOTS: [&'static str; 3] = ["crate", "self", "super"];

    pub(super) fn of_root(root: &Name) -> Self {
        Some(root)
            .filter(|root| Self::HOME_ROOTS.iter().all(|home_root| **root != Name::from(*home_root)))
            .map_or(Self::Home, |root| Self::Crate(CrateName::from(root)))
    }

    pub(super) fn routed<T>(&self, home: impl FnOnce() -> T, owned: impl FnOnce(&CrateName) -> T) -> T {
        match self {
            Self::Home => home(),
            Self::Crate(named) => owned(named),
        }
    }
}
