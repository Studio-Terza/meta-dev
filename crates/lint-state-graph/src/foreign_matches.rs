use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use graph::{Held, ItemId, Mention, Model, Reach, Region};
use lint_core::{Headline, Report, Violation};
use lint_file::ModelLint;
use lint_layout::Name;

use crate::finding::Finding;

#[derive(Debug)]
pub struct ForeignMatches;

#[derive(Clone, Copy, Debug)]
struct Declared<'model> {
    path: &'model Path,
    name: &'model Name,
}

impl ModelLint for ForeignMatches {
    fn report(model: &Model) -> Report<'_> {
        let sums: BTreeSet<ItemId> = model
            .holdings()
            .filter(|placed| placed.entity().held() == Held::EnumField)
            .filter_map(|placed| placed.entity().declares())
            .collect();
        let enums: BTreeMap<ItemId, Declared<'_>> = model
            .items()
            .filter_map(|placed| {
                let id = sums.get(&placed.entity().declared()).copied()?;
                let item = model.declared(id)?;
                Some((id, Declared { path: <&Path>::from(placed.site()), name: item.as_ref() }))
            })
            .collect();
        let foreign = model
            .uses()
            .filter(|used| used.entity().mention() == Mention::Matched)
            .filter(|used| AsRef::<Region>::as_ref(used.entity()).reach() == Reach::Shipped)
            .filter_map(|used| {
                let declared = enums.get(&used.entity().resolved().member()?.owner())?;
                Some(used.site())
                    .filter(|site| <&Path>::from(*site) != declared.path)
                    .map(|site| Violation::new(site, Finding::ForeignMatch { enum_name: declared.name }))
            })
            .collect();
        Report::new(Headline::new("state graph violated"), foreign)
    }
}
