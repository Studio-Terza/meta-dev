use lint_layout::Name;
use lint_placement::Level;
use lint_placement::Nest;

use super::ModulePath;

pub struct InsideSibling;

impl InsideSibling {
    pub fn home<'file>(user: &ModulePath<'file>, module: &ModulePath<'_>) -> Option<Nest<'file>> {
        let depth = <&[Name]>::from(*AsRef::<Nest<'_>>::as_ref(module)).len();
        let inside = *AsRef::<Nest<'file>>::as_ref(user);
        let held = AsRef::<Nest<'_>>::as_ref(module)
            .parent()
            .filter(|parent| parent.level() != Level::Workspace)
            .filter(|parent| inside.lineage().any(|outer| outer == *parent))
            .filter(|_| <&[Name]>::from(inside).len() >= depth);
        let outer = inside.lineage().find(|outer| <&[Name]>::from(*outer).len() == depth);
        held.map(|_| outer.unwrap_or(inside))
    }
}
