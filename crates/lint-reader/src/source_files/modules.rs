mod child;
mod declaration;
mod redirects;

use std::io;
use std::path::Path;

use syn::Item;

use lint_core::Walk;
use lint_layout::Failure;
use lint_layout::Name;

pub use child::Child;
use declaration::Declaration;
use redirects::Redirects;

#[derive(Debug)]
pub struct Modules {
    children: Vec<Child>,
    tests: Vec<Child>,
    failures: Vec<Failure>,
}

impl Modules {
    const SOURCE_EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    #[must_use]
    pub fn declared_in(file: &Path, children_directory: &Path, items: &[Item]) -> Modules {
        let file_directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
        let entered = Self::declarations(items, children_directory, Walk::Entered);
        let skipped = Self::declarations(items, children_directory, Walk::Skipped);
        let tests = skipped
            .iter()
            .flat_map(|declaration| Self::resolve(&file_directory, declaration).unwrap_or_default())
            .collect();
        entered.into_iter().fold(
            Modules { children: Vec::new(), tests, failures: Vec::new() },
            |modules, declaration| match Self::resolve(&file_directory, &declaration) {
                Ok(found) => Modules { children: modules.children.into_iter().chain(found).collect(), ..modules },
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

    fn resolve(file_directory: &Path, declaration: &Declaration<'_>) -> Result<Vec<Child>, Failure> {
        let own = declaration.directory().join(Name::from(&declaration.module().ident));
        let explicit = Redirects::explicit(declaration.module()).map(|path| file_directory.join(path));
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
        let primary = choices.into_iter().find(Child::exists);
        let found: Vec<Child> = primary.iter().cloned().collect();
        let conditional = Redirects::conditional(file_directory, declaration.module());
        let found = conditional.into_iter().filter(Child::exists).fold(found, |found, child| {
            if found.contains(&child) { found } else { found.into_iter().chain([child]).collect() }
        });
        if found.is_empty() {
            Err(Failure::Io { path: expected, source: io::Error::from(io::ErrorKind::NotFound) })
        } else {
            Ok(found)
        }
    }
}
