use std::path::{Component, Path, PathBuf};

use syntax::ItemMod;

use super::PublicApiTests;
use lint_core::{Site, SourceFile, Syntax, Violation};
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Contribute, Contribution, Facts, Kind, Scope};

use super::test_code::Tested;

#[derive(Debug)]
pub struct PathReach<'file> {
    path: &'file Path,
    directory: PathBuf,
    source: PathBuf,
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

#[derive(Debug)]
pub struct Reached {
    target: PathBuf,
    line: LineNumber,
}

impl Reached {
    fn under(self, name: Option<&Name>) -> Reached {
        let directory = name.map_or_else(PathBuf::new, |within| AsRef::<Path>::as_ref(within).to_path_buf());
        Reached { target: directory.join(self.target), line: self.line }
    }
}

impl Contribute for PathReach<'_> {
    type Open = ();
    type Part = Vec<Reached>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemMod)] }
    }

    fn off_item_mod(
        &self,
        module: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        inner: Contribution<(), Vec<Reached>>,
    ) -> Vec<Reached> {
        let own = Self::target(module).map(|target| Reached { target, line: LineNumber::from(module.ident.span()) });
        let nested = inner.concluded(|(), reached| reached);
        own.into_iter().chain(nested.into_iter().map(|within| within.under(facts.name()))).collect()
    }
}

impl<'file> PathReach<'file> {
    pub(crate) fn tested(self, reached: Vec<Reached>) -> Tested<'file> {
        let path = self.path;
        let found = reached
            .into_iter()
            .filter(|reach| Self::lexical(&self.directory.join(&reach.target)).starts_with(&self.source))
            .map(|reach| Violation::new(Site::new(path, reach.line), PublicApiTests::REACH));
        Tested::reached(found.collect(), path)
    }
}
