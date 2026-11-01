mod definitions;

use std::collections::BTreeSet;

use lint_core::SourceFile;
use lint_layout::Name;

use definitions::Definitions;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Wrappers {
    plain: BTreeSet<Name>,
}

impl Wrappers {
    pub fn settled_in(files: &[&SourceFile]) -> Wrappers {
        let definitions = Definitions::collected_from(files);
        std::iter::successors(Some(Wrappers::default()), |settled| {
            Some(settled.widened(&definitions)).filter(|wider| wider != settled)
        })
        .last()
        .unwrap_or_default()
    }

    pub fn holds_text(&self, name: &Name) -> bool {
        self.plain.contains(name)
    }

    fn widened(&self, definitions: &Definitions) -> Wrappers {
        Wrappers { plain: definitions.plain_among(self) }
    }
}
