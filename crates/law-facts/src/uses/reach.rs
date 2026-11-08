use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Reach {
    Home,
    Crate(CrateName),
}

impl Reach {
    const HOME_ROOTS: [&'static str; 3] = ["crate", "self", "super"];

    pub(super) fn of_root(root: &Name) -> Reach {
        Some(root)
            .filter(|candidate| Self::HOME_ROOTS.iter().all(|home_root| **candidate != Name::from(*home_root)))
            .map_or(Reach::Home, |foreign| Reach::Crate(CrateName::from(foreign)))
    }

    pub(super) fn routed<T>(&self, home: impl FnOnce() -> T, owned: impl FnOnce(&CrateName) -> T) -> T {
        match self {
            Reach::Home => home(),
            Reach::Crate(named) => owned(named),
        }
    }
}
