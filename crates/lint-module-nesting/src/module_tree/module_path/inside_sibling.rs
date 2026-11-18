use lint_layout::Name;
use lint_placement::Level;
use lint_placement::Lineage;
use lint_placement::Nest;
use std::path::Path;

use super::ModulePath;

pub struct InsideSibling;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Siding {
    Inside,
    Elsewhere,
}

impl InsideSibling {
    pub fn placed(this: &ModulePath<'_>, module: &ModulePath<'_>) -> Siding {
        let length = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        AsRef::<Nest<'_>>::as_ref(module)
            .parent()
            .filter(|parent| parent.level() != Level::Workspace)
            .filter(|parent| parent.relation(*AsRef::<Nest<'_>>::as_ref(this)) == Lineage::Holds)
            .filter(|_| <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(this)).len() >= length)
            .map_or(Siding::Elsewhere, |_| Siding::Inside)
    }

    pub fn home<'file>(user: &ModulePath<'file>, module: &ModulePath<'_>) -> ModulePath<'file> {
        let nest = *AsRef::<Nest<'_>>::as_ref(user);
        let depth = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        let outer = nest.lineage().find(|outer| <&[Name]>::from(*outer).len() == depth);
        ModulePath::new(Path::new(""), outer.unwrap_or(nest))
    }
}
