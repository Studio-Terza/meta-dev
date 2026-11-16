use syn::punctuated::Punctuated;
use syn::{Meta, Token};

#[derive(Debug)]
pub struct Applied;

impl Applied {
    const CONDITIONAL: &'static str = "cfg_attr";

    #[must_use]
    pub fn metas(meta: &Meta) -> Vec<Meta> {
        let list = meta.require_list().ok().filter(|list| list.path.is_ident(Self::CONDITIONAL));
        let nested = list.and_then(|found| found.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok());
        nested.map_or_else(|| vec![meta.clone()], |arguments| arguments.iter().skip(1).flat_map(Self::metas).collect())
    }
}
