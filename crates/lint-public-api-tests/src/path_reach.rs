use std::path::{Component, Path, PathBuf};

use syn::ItemMod;
use syn::visit::{self, Visit};

use lint_core::{SourceFile, Syntax};
use lint_layout::LineNumber;
use lint_layout::Name;

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
        let mut reach = Self::beside(file);
        reach.visit_file(file.syntax());
        reach.lines
    }

    fn beside(file: &SourceFile) -> Self {
        let package = AsRef::<Path>::as_ref(file.manifest_file()).parent().map(Path::to_path_buf).unwrap_or_default();
        Self {
            directory: file.path().parent().map(Path::to_path_buf).unwrap_or_default(),
            source: Self::lexical(&package.join(Self::SOURCE_DIRECTORY)),
            lines: Vec::new(),
        }
    }

    fn lexical(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), Self::stepped)
    }

    fn stepped(folded: PathBuf, component: Component) -> PathBuf {
        match component {
            Component::ParentDir => folded.file_name().map_or_else(
                || folded.join(Component::ParentDir),
                |_| folded.parent().map(Path::to_path_buf).unwrap_or_default(),
            ),
            Component::CurDir => folded,
            other => folded.join(other),
        }
    }

    fn target(module: &ItemMod) -> Option<PathBuf> {
        module
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::PATH_ATTRIBUTE))
            .filter_map(|attribute| attribute.meta.require_name_value().ok())
            .find_map(|pair| Syntax::string_literal(&pair.value))
            .map(|path| PathBuf::from(path.value()))
    }
}

impl<'ast> Visit<'ast> for PathReach {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.lines.extend(
            Self::target(module)
                .filter(|target| Self::lexical(&self.directory.join(target)).starts_with(&self.source))
                .map(|_| LineNumber::from(module.ident.span())),
        );
        self.directory.push(Name::from(&module.ident));
        visit::visit_item_mod(self, module);
        self.directory.pop();
    }
}
