mod finding;
mod kinds;
mod texts;

use std::collections::BTreeSet;

use code_model::{Held, Holding, ItemId, Model, Placed, Site, TypeRef};
use lint_core::{Headline, ModelLint, Report, Violation};
use lint_layout::Name;

use finding::Finding;
use kinds::{Derive, Kinds};
use texts::{Spots, Texts};

#[derive(Debug)]
pub struct NoPlaintext;

#[derive(Clone, Copy, Debug)]
struct Texted<'model> {
    owner: ItemId,
    name: &'model Name,
    site: Site<'model>,
}

impl ModelLint for NoPlaintext {
    fn report(model: &Model) -> Report<'_> {
        let kinds = Kinds::read(model);
        let texts = Texts::settled(model, &kinds);
        let texted: Vec<Texted<'_>> =
            model.holdings().filter_map(|placed| Self::texted(model, &texts, placed)).collect();
        let owners: BTreeSet<ItemId> = texted.iter().map(|field| field.owner).collect();
        let plain = texted.iter().filter_map(|field| {
            kinds.plain(field.owner).map(|_| Violation::new(field.site, Finding::Plain(field.name)))
        });
        let unresolved = kinds.unresolved().iter().filter_map(|derive: &Derive<'_>| {
            owners.get(&derive.subject()).map(|_| Violation::new(derive.site(), Finding::Unresolved(derive.name())))
        });
        let mut found: Vec<Violation<'_>> = plain.chain(unresolved).collect();
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
    fn texted<'model>(
        model: &'model Model,
        texts: &Texts,
        placed: Placed<'model, &'model Holding>,
    ) -> Option<Texted<'model>> {
        let holding = Self::field(placed.entity())?;
        let owner = holding.declares()?;
        let ty: &TypeRef = holding.as_ref();
        texts.text(&model.expanded(ty))?;
        let site = placed.site();
        let spotted = Spots::first(ty).map_or(site, |position| Site::new(site.path(), position.line()));
        Some(Texted { owner, name: holding.as_ref(), site: spotted })
    }

    fn field(holding: &Holding) -> Option<&Holding> {
        match holding.held() {
            Held::StructField | Held::EnumField | Held::NewtypeField => Some(holding),
            Held::UnionField | Held::StructParameter | Held::Alias => None,
        }
    }
}
