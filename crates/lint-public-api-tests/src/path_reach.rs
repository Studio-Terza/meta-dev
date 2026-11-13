use std::path::{Component, Path, PathBuf};

use syn::ItemMod;

use super::PublicApiTests;
use lint_core::{Site, SourceFile, Syntax, Violation};
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Finds, Kind, Watch};

#[derive(Debug)]
pub(crate) struct PathReach<'file> {
    path: &'file Path,
    directory: PathBuf,
    source: PathBuf,
    lines: Vec<LineNumber>,
}

impl<'file> PathReach<'file> {
    const PATH_ATTRIBUTE: &'static str = "path";
    const SOURCE_DIRECTORY: &'static str = "src";

    pub(crate) fn beside(file: &'file SourceFile) -> PathReach<'file> {
        let package = AsRef::<Path>::as_ref(file.head().manifest_file()).parent().map_or_default(Path::to_path_buf);
        PathReach {
            path: file.head().path(),
            directory: file.head().path().parent().map_or_default(Path::to_path_buf),
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
                |_| folded.parent().map_or_default(Path::to_path_buf),
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

impl<'ast> Watch<'ast> for PathReach<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        self.lines.extend(
            Self::target(module)
                .filter(|target| Self::lexical(&self.directory.join(target)).starts_with(&self.source))
                .map(|_| LineNumber::from(module.ident.span())),
        );
        self.directory.push(Name::from(&module.ident));
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.directory.pop();
    }
}

impl<'file> Finds<'file> for PathReach<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.path;
        self.lines.into_iter().map(|line| Violation::new(Site::new(path, line), PublicApiTests::REACH)).collect()
    }
}
