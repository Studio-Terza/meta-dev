use std::str::FromStr;

use syn::Signature;
use type_facts::MethodShape;

use crate::boundary::QueryFailure;
use crate::query::score::Score;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Role(MethodShape);

impl Role {
    #[must_use]
    pub fn agreement(&self, other: &MethodShape) -> Score {
        let wanted = &self.0;
        let parts = [
            wanted.receiver() == other.receiver(),
            wanted.takes().len() == other.takes().len(),
            wanted.takes() == other.takes(),
            wanted.returned() == other.returned(),
        ];
        Score::new(parts.into_iter().filter(|agrees| *agrees).count())
    }
}

impl AsRef<MethodShape> for Role {
    fn as_ref(&self) -> &MethodShape {
        &self.0
    }
}

impl FromStr for Role {
    type Err = QueryFailure;

    fn from_str(text: &str) -> Result<Role, QueryFailure> {
        syn::parse_str::<Signature>(text)
            .map(|signature| Role(MethodShape::of_signature(&signature, None)))
            .map_err(|failure| QueryFailure::Role { text: text.to_owned(), reason: failure.to_string() })
    }
}
