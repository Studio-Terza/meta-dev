use std::path::{Component, Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{Expr, ExprLit, ItemMod, Lit};

use lint_core::SourceFile;
use lint_layout::{LineNumber, Name};

#[derive(Debug)]
pub(crate) struct PathReach {
    directory: PathBuf,
    source: PathBuf,
    lines: Vec<LineNumber>,
}

impl PathReach {
    const PATH_ATTRIBUTE: &'static str = "path";
    const SOURCE_DIRECTORY: &'static str = "src";

    pub(crate) fn lines_in(file: &SourceFile) -> Vec<LineNumber> {
        let mut reach = PathReach::beside(file);
        reach.visit_file(file.syntax());
        reach.lines
    }

    fn beside(file: &SourceFile) -> PathReach {
        let package = file.manifest_file().parent().map(Path::to_path_buf).unwrap_or_default();
        PathReach {
            directory: file.path().parent().map(Path::to_path_buf).unwrap_or_default(),
            source: Self::lexical(&package.join(Self::SOURCE_DIRECTORY)),
            lines: Vec::new(),
        }
    }

    fn lexical(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |folded, component| match component {
            Component::ParentDir if folded.file_name().is_some() => {
                folded.parent().map(Path::to_path_buf).unwrap_or_default()
            }
            Component::CurDir => folded,
            other => folded.join(other),
        })
    }

    fn target(module: &ItemMod) -> Option<PathBuf> {
        module.attrs.iter().filter(|attribute| attribute.path().is_ident(Self::PATH_ATTRIBUTE)).find_map(|attribute| {
            match &attribute.meta.require_name_value().ok()?.value {
                Expr::Lit(ExprLit { lit: Lit::Str(path), .. }) => Some(PathBuf::from(path.value())),
                _ => None,
            }
        })
    }
}

impl<'ast> Visit<'ast> for PathReach {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.lines.extend(
            Self::target(module)
                .filter(|target| Self::lexical(&self.directory.join(target)).starts_with(&self.source))
                .map(|_| LineNumber::from(module.ident.span())),
        );
        let outer = self.directory.clone();
        self.directory = outer.join(Name::from(&module.ident));
        visit::visit_item_mod(self, module);
        self.directory = outer;
    }
}
