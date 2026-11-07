mod finding;
mod kinds;
mod texts;

use code_model::{Held, Holding, Model, Placed, Site, TypeRef};
use lint_core::{Headline, ModelLint, Report, Violation};

use finding::Finding;
use kinds::Kinds;
use texts::{Spots, Texts};

#[derive(Debug)]
pub struct NoPlaintext;

impl ModelLint for NoPlaintext {
    fn report(model: &Model) -> Report<'_> {
        let kinds = Kinds::read(model);
        let texts = Texts::settled(model, &kinds);
        let mut found: Vec<Violation<'_>> =
            model.holdings().filter_map(|placed| Self::flagged(model, &kinds, &texts, placed)).collect();
        found.sort_by(|left, right| left.site().cmp(right.site()));
        Report::new(
            Headline::new(
                "data that enters from a file holds no free text; split it into typed parts, or give it a newtype that validates",
            ),
            found,
        )
    }
}

impl NoPlaintext {
    const FIELDS: [Held; 3] = [Held::StructField, Held::EnumField, Held::NewtypeField];

    fn flagged<'model>(
        model: &'model Model,
        kinds: &Kinds,
        texts: &Texts,
        placed: Placed<'model, &'model Holding>,
    ) -> Option<Violation<'model>> {
        let holding = placed.entity();
        Self::FIELDS.iter().find(|held| **held == holding.held())?;
        kinds.plain_owner(holding)?;
        let ty: &TypeRef = holding.as_ref();
        texts.text(&model.expanded(ty))?;
        let site = placed.site();
        let spotted = Spots::first(ty).map_or(site, |position| Site::new(site.path(), position.line()));
        Some(Violation::new(spotted, Finding::new(holding.as_ref())))
    }
}
