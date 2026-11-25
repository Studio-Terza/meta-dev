use lint_layout::Name;
use lint_placement::Level;
use lint_placement::Nest;
use std::path::Path;

use super::ModulePath;

pub struct InsideSibling;

impl InsideSibling {
    pub fn holder<'module>(this: &ModulePath<'_>, module: &ModulePath<'module>) -> Option<Nest<'module>> {
        let length = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        let inside = *AsRef::<Nest<'_>>::as_ref(this);
        AsRef::<Nest<'_>>::as_ref(module)
            .parent()
            .filter(|parent| parent.level() != Level::Workspace)
            .filter(|parent| inside.lineage().any(|outer| outer == *parent))
            .filter(|_| <&[Name]>::from(inside).len() >= length)
    }

    pub fn home<'file>(user: &ModulePath<'file>, module: &ModulePath<'_>) -> ModulePath<'file> {
        let nest = *AsRef::<Nest<'_>>::as_ref(user);
        let depth = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        let outer = nest.lineage().find(|outer| <&[Name]>::from(*outer).len() == depth);
        ModulePath::new(Path::new(""), outer.unwrap_or(nest))
    }
}
