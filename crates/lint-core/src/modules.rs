use std::io;
use std::path::{Path, PathBuf};

use syn::{Expr, ExprLit, Item, ItemMod, Lit};

use crate::failure::Failure;
use crate::walk::Walk;

#[derive(Debug)]
pub struct Modules {
    file_directory: PathBuf,
    found: Vec<Child>,
    missing: Vec<Failure>,
}

#[derive(Debug)]
pub struct Child {
    path: PathBuf,
    children_directory: PathBuf,
}

impl Child {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn children_directory(&self) -> &Path {
        &self.children_directory
    }
}

impl Modules {
    #[must_use]
    pub fn declared_in(file: &Path, children_directory: &Path, items: &[Item]) -> Modules {
        let mut modules = Modules {
            file_directory: file.parent().map(Path::to_path_buf).unwrap_or_default(),
            found: Vec::new(),
            missing: Vec::new(),
        };
        modules.collect(items, children_directory);
        modules
    }

    pub fn take_children(&mut self) -> Vec<Child> {
        std::mem::take(&mut self.found)
    }

    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.missing)
    }

    fn collect(&mut self, items: &[Item], directory: &Path) {
        for item in items {
            if let Item::Mod(module) = item {
                if Walk::outside_tests(&module.attrs) == Walk::Skipped {
                    continue;
                }
                match &module.content {
                    Some((_, inner)) => self.collect(inner, &directory.join(module.ident.to_string())),
                    None => self.declare(module, directory),
                }
            }
        }
    }

    fn explicit_path(module: &ItemMod) -> Option<String> {
        module.attrs.iter().filter(|attribute| attribute.path().is_ident("path")).find_map(|attribute| match &attribute
            .meta
            .require_name_value()
            .ok()?
            .value
        {
            Expr::Lit(ExprLit { lit: Lit::Str(path), .. }) => Some(path.value()),
            _ => None,
        })
    }

    fn declare(&mut self, module: &ItemMod, directory: &Path) {
        let name = module.ident.to_string();
        let choices: Vec<Child> = match Self::explicit_path(module) {
            Some(path) => {
                let file = self.file_directory.join(path);
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                vec![Child { path: file, children_directory: children }]
            }
            None => vec![
                Child { path: directory.join(format!("{name}.rs")), children_directory: directory.join(&name) },
                Child { path: directory.join(&name).join("mod.rs"), children_directory: directory.join(&name) },
            ],
        };
        match choices.iter().position(|choice| choice.path.is_file()) {
            Some(index) => self.found.extend(choices.into_iter().nth(index)),
            None => self.missing.push(Failure::Io {
                path: choices.into_iter().next().map(|choice| choice.path).unwrap_or_default(),
                source: io::Error::from(io::ErrorKind::NotFound),
            }),
        }
    }
}
