use body_lifts::Branching;
use graph::{Entry, Placed, Reach, Region, Writing};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use newtype::Counter;

#[derive(Clone, Debug)]
pub(crate) struct Function<'model> {
    krate: CrateName,
    name: Name,
    site: Site<'model>,
    score: Branching,
}

impl<'model> Function<'model> {
    pub(crate) fn scored(placed: Placed<'model, &'model graph::Function>) -> Option<Function<'model>> {
        let function = placed.entity();
        let region: &Region = function.as_ref();
        Some(region.reach()).filter(|reach| *reach == Reach::Shipped)?;
        Some(region.writing()).filter(|writing| *writing == Writing::Written)?;
        Some(function.entry()).filter(|entry| *entry != Entry::Test)?;
        let definition = function.definition()?;
        let score = definition.branching().plus(Branching::from(definition.propagated()));
        Some(Function {
            krate: CrateName::from(placed.krate()),
            name: AsRef::<Name>::as_ref(function).clone(),
            site: placed.site(),
            score,
        })
    }

    pub(crate) fn score(&self) -> Branching {
        self.score
    }
}

impl AsRef<CrateName> for Function<'_> {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl AsRef<Name> for Function<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'model> AsRef<Site<'model>> for Function<'model> {
    fn as_ref(&self) -> &Site<'model> {
        &self.site
    }
}
