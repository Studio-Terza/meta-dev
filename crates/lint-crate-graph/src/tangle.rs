use std::fmt;

use graph::{ItemId, Model, References};
use lint_core::{Bound, Fix, Site, Violation};
use lint_layout::{Length, Name};
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Tangle(Vec<Name>);

impl Tangle {
    const ITEMS: Bound = Bound::Most(40);

    pub(crate) fn flagged<'model>(
        model: &'model Model,
        references: &'model References,
        members: Vec<&ItemId>,
    ) -> Option<Violation<'model>> {
        let root = members.iter().find_map(|member| references.root(**member))?;
        let mut names: Vec<Name> = members
            .into_iter()
            .filter_map(|member| model.declared(*member))
            .map(|item| AsRef::<Name>::as_ref(item).clone())
            .collect();
        names.sort();
        let tangle = Tangle(names);
        Self::ITEMS
            .allows(Length::new(tangle.0.len()))
            .map_or(Some(Violation::new(Site::of_file(root), tangle)), |_| None)
    }
}

impl fmt::Display for Tangle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "cycle of {} items: {}", self.0.len(), names.join(" "))
    }
}

impl lint_core::Finding for Tangle {
    fn fix(&self) -> Fix {
        Fix::new("break the cycle with a standard trait, else a generic over an existing trait, else a derive")
    }
}
