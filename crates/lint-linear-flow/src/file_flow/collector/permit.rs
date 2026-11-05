use syn::{Block, ItemImpl, TypePath};

use lint_core::Syntax;

use super::zone::Zone;
use crate::function_flow::Root;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permit {
    BoolConversion,
    Plain,
}

impl Permit {
    const FROM: &'static str = "From";
    const BOOL: &'static str = "bool";

    pub fn within(zone: Zone, item: &ItemImpl) -> Permit {
        match zone {
            Zone::Boundary => Self::converting(Self::bool_source(item)),
            Zone::Elsewhere => Permit::Plain,
        }
    }

    pub fn root(self, body: &Block) -> Root<'_> {
        match self {
            Permit::BoolConversion => Root::converting(body),
            Permit::Plain => Root::from(body),
        }
    }

    fn converting(source: Option<&TypePath>) -> Permit {
        match source {
            Some(_) => Permit::BoolConversion,
            None => Permit::Plain,
        }
    }

    fn bool_source(item: &ItemImpl) -> Option<&TypePath> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|segment| segment.ident == Self::FROM)
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|angled| angled.args.first())
            .and_then(Syntax::type_argument)
            .and_then(Syntax::type_path)
            .filter(|source| source.path.is_ident(Self::BOOL))
    }
}
