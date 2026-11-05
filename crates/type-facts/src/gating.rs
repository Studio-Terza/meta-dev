use syn::Attribute;

use lint_core::Walk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gating {
    Shipped,
    Tested,
}

impl Gating {
    #[must_use]
    pub fn within(self, attributes: &[Attribute]) -> Gating {
        self.joined(Gating::marked_by(attributes))
    }

    pub(crate) fn pick<Found>(self, shipped: Found, tested: Found) -> Found {
        match self {
            Gating::Shipped => shipped,
            Gating::Tested => tested,
        }
    }

    fn marked_by(attributes: &[Attribute]) -> Gating {
        match Walk::outside_tests(attributes) {
            Walk::Skipped => Gating::Tested,
            Walk::Entered => Gating::Shipped,
        }
    }

    fn joined(self, inner: Gating) -> Gating {
        self.pick(inner, Gating::Tested)
    }
}
