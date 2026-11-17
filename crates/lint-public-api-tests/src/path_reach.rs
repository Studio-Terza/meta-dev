use std::path::{Component, Path, PathBuf};

use syntax::ItemMod;

use super::PublicApiTests;
use lint_core::{Site, SourceFile, Syntax, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Facts, Fold, Kind, Visit};

use super::test_code::Tested;

#[derive(Debug)]
pub struct PathReach<'file> {
    path: &'file Path,
    directory: PathBuf,
    source: PathBuf,
}

#[derive(Debug, Default)]
pub struct Reached {
    directory: Option<PathBuf>,
    lines: Vec<LineNumber>,
}

impl<'file> PathReach<'file> {
    const PATH_ATTRIBUTE: &'static str = "path";
    const SOURCE_DIRECTORY: &'static str = "src";

    pub(crate) fn beside(file: &'file SourceFile) -> PathReach<'file> {
        let package = AsRef::<Path>::as_ref(file.head().manifest_file()).parent().map_or_default(Path::to_path_buf);
        PathReach {
            path: AsRef::<Path>::as_ref(file.head()),
            directory: AsRef::<Path>::as_ref(file.head()).parent().map_or_default(Path::to_path_buf),
            source: Self::lexical(&package.join(Self::SOURCE_DIRECTORY)),
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

impl Fold for PathReach<'_> {
    type Found = Reached;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod]
    }

    fn on_item_mod(&self, found: Reached, module: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Reached> {
        let directory = found.directory.unwrap_or_else(|| self.directory.clone());
        let reaching = Self::target(module)
            .filter(|target| Self::lexical(&directory.join(target)).starts_with(&self.source))
            .map(|_| LineNumber::from(module.ident.span()));
        let inner = facts.name().into_iter().fold(directory, |outer, name| outer.join(name));
        Visit::descending(Reached { directory: Some(inner), lines: found.lines.into_iter().chain(reaching).collect() })
    }

    fn off_item_mod(&self, found: Reached, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Reached {
        let directory = found.directory.clone().unwrap_or_else(|| self.directory.clone());
        let outer = directory.parent().map(Path::to_path_buf);
        Reached { directory: outer, ..found }
    }
}

impl<'file> PathReach<'file> {
    pub(crate) fn tested(&self, reached: Reached) -> Tested<'file> {
        let path = self.path;
        let found = reached.lines.into_iter().map(|line| Violation::new(Site::new(path, line), PublicApiTests::REACH));
        Tested::reached(found.collect(), path)
    }
}
