use lint_layout::Name;
use lint_placement::Level;
use lint_placement::Nest;
use std::path::Path;

use super::ModulePath;

pub struct InsideSibling;

impl InsideSibling {
    pub fn holds(this: &ModulePath<'_>, module: &ModulePath<'_>) -> Option<()> {
        let length = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        AsRef::<Nest<'_>>::as_ref(module)
            .parent()
            .filter(|parent| parent.level() != Level::Workspace)
            .and_then(|parent| parent.holding(*AsRef::<Nest<'_>>::as_ref(this)))
            .filter(|()| <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(this)).len() >= length)
    }

    pub fn home<'file>(user: &ModulePath<'file>, module: &ModulePath<'_>) -> ModulePath<'file> {
        let nest = *AsRef::<Nest<'_>>::as_ref(user);
        let depth = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        let outer = nest.lineage().find(|outer| <&[Name]>::from(*outer).len() == depth);
        ModulePath::new(Path::new(""), outer.unwrap_or(nest))
    }
}
