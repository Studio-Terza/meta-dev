mod child;
mod declaration;

use std::io;
use std::path::{Path, PathBuf};

use syn::{Expr, ExprLit, Item, ItemMod, Lit};

use crate::failure::Failure;
use crate::name::Name;
use crate::walk::Walk;

pub use child::Child;
use declaration::Declaration;

#[derive(Debug)]
pub struct Modules {
    children: Vec<Child>,
    tests: Vec<Child>,
    failures: Vec<Failure>,
}

impl Modules {
    const SOURCE_EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";
    const PATH_ATTRIBUTE: &'static str = "path";

    #[must_use]
    pub fn declared_in(file: &Path, children_directory: &Path, items: &[Item]) -> Modules {
        let file_directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
        let entered = Self::declarations(items, children_directory, Walk::Entered);
        let skipped = Self::declarations(items, children_directory, Walk::Skipped);
        let tests = skipped.iter().filter_map(|declaration| Self::resolve(&file_directory, declaration).ok()).collect();
        entered.into_iter().fold(
            Modules { children: Vec::new(), tests, failures: Vec::new() },
            |modules, declaration| match Self::resolve(&file_directory, &declaration) {
                Ok(child) => Modules { children: modules.children.into_iter().chain([child]).collect(), ..modules },
                Err(failure) => {
                    Modules { failures: modules.failures.into_iter().chain([failure]).collect(), ..modules }
                }
            },
        )
    }

    pub fn take_children(&mut self) -> Vec<Child> {
        std::mem::take(&mut self.children)
    }

    #[must_use]
    pub fn tests(&self) -> &[Child] {
        &self.tests
    }

    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.failures)
    }

    fn declarations<'items>(items: &'items [Item], directory: &Path, wanted: Walk) -> Vec<Declaration<'items>> {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(module) => Some(module),
                _ => None,
            })
            .flat_map(|module| match (&module.content, Walk::outside_tests(&module.attrs)) {
                (Some((_, inner)), Walk::Entered) => {
                    Self::declarations(inner, &directory.join(Name::from(&module.ident)), wanted)
                }
                (None, walk) if walk == wanted => vec![Declaration::new(module, directory.to_path_buf())],
                _ => Vec::new(),
            })
            .collect()
    }

    fn explicit_path(module: &ItemMod) -> Option<PathBuf> {
        module.attrs.iter().filter(|attribute| attribute.path().is_ident(Self::PATH_ATTRIBUTE)).find_map(|attribute| {
            match &attribute.meta.require_name_value().ok()?.value {
                Expr::Lit(ExprLit { lit: Lit::Str(path), .. }) => Some(PathBuf::from(path.value())),
                _ => None,
            }
        })
    }

    fn resolve(file_directory: &Path, declaration: &Declaration<'_>) -> Result<Child, Failure> {
        let own = declaration.directory().join(Name::from(&declaration.module().ident));
        let explicit = Self::explicit_path(declaration.module()).map(|path| file_directory.join(path));
        let expected = explicit.clone().unwrap_or_else(|| own.with_extension(Self::SOURCE_EXTENSION));
        let choices = match explicit {
            Some(file) => {
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                vec![Child::new(file, children)]
            }
            None => vec![
                Child::new(own.with_extension(Self::SOURCE_EXTENSION), own.clone()),
                Child::new(own.join(Self::MODULE_FILE), own),
            ],
        };
        choices
            .into_iter()
            .find(Child::exists)
            .ok_or_else(|| Failure::Io { path: expected, source: io::Error::from(io::ErrorKind::NotFound) })
    }
}
