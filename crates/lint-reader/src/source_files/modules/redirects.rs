use std::path::{Path, PathBuf};

use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprLit, ItemMod, Lit, Meta, Token};

use super::child::Child;

#[derive(Debug)]
pub struct Redirects;

impl Redirects {
    const PATH_ATTRIBUTE: &'static str = "path";
    const CONDITIONAL_ATTRIBUTE: &'static str = "cfg_attr";

    #[must_use]
    pub fn explicit(module: &ItemMod) -> Option<PathBuf> {
        module.attrs.iter().find_map(|attribute| Self::literal(&attribute.meta))
    }

    #[must_use]
    pub fn conditional(file_directory: &Path, module: &ItemMod) -> Vec<Child> {
        module
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::CONDITIONAL_ATTRIBUTE))
            .flat_map(Self::conditional_paths)
            .map(|path| file_directory.join(path))
            .map(|file| {
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                Child::new(file, children)
            })
            .collect()
    }

    fn literal(meta: &Meta) -> Option<PathBuf> {
        match &meta.require_name_value().ok().filter(|_| meta.path().is_ident(Self::PATH_ATTRIBUTE))?.value {
            Expr::Lit(ExprLit { lit: Lit::Str(path), .. }) => Some(PathBuf::from(path.value())),
            _ => None,
        }
    }

    fn conditional_paths(attribute: &Attribute) -> Vec<PathBuf> {
        attribute
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .into_iter()
            .flatten()
            .skip(1)
            .filter_map(|meta| Self::literal(&meta))
            .collect()
    }
}
