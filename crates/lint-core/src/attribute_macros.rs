use syn::punctuated::Punctuated;
use syn::{Meta, Path, Token};

#[derive(Debug)]
pub struct AttributeMacros;

impl AttributeMacros {
    const DERIVE: &'static str = "derive";
    const CONDITIONAL: &'static str = "cfg_attr";

    #[must_use]
    pub fn expanded(meta: &Meta) -> Vec<Path> {
        match Listing::from(meta.path()) {
            Listing::Derive => meta
                .require_list()
                .ok()
                .and_then(|list| list.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
                .into_iter()
                .flatten()
                .collect(),
            Listing::Conditional => meta
                .require_list()
                .ok()
                .and_then(|list| list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok())
                .into_iter()
                .flatten()
                .skip(1)
                .flat_map(|nested| Self::expanded(&nested))
                .collect(),
            Listing::Other => vec![meta.path().clone()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Listing {
    Derive,
    Conditional,
    Other,
}

impl Listing {
    const NAMED: [(&'static str, Listing); 2] =
        [(AttributeMacros::DERIVE, Listing::Derive), (AttributeMacros::CONDITIONAL, Listing::Conditional)];
}

impl From<&Path> for Listing {
    fn from(path: &Path) -> Listing {
        Listing::NAMED.iter().find(|(name, _)| path.is_ident(name)).map_or(Listing::Other, |(_, listing)| *listing)
    }
}
