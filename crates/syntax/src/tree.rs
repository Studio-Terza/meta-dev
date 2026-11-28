use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use invariants::Ignoring;
use syn::{Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue};

use crate::failure::Failure;
use crate::ignored::Ignored;

#[derive(Debug)]
pub struct Tree {
    given: PathBuf,
    canonical: PathBuf,
    texts: BTreeMap<PathBuf, String>,
}

impl Tree {
    const CURRENT: &'static str = ".";
    pub const MANIFEST: &'static str = "Cargo.toml";
    pub const BUILD_OUTPUT: &'static str = "target";
    pub const SOURCE_EXTENSION: &'static str = "rs";
    const PATH_ATTRIBUTE: &'static str = "path";

    #[must_use]
    pub fn new(given: PathBuf, canonical: PathBuf, texts: BTreeMap<PathBuf, String>) -> Tree {
        Tree { given, canonical, texts }
    }

    #[must_use]
    pub fn with(self, texts: BTreeMap<PathBuf, String>) -> Tree {
        Tree { texts: self.texts.into_iter().chain(texts).collect(), ..self }
    }

    pub(crate) fn text(&self, path: &Path) -> Option<&str> {
        self.texts.get(path).map(AsRef::as_ref)
    }

    pub(crate) fn below<'tree>(&'tree self, directory: &'tree Path) -> impl Iterator<Item = &'tree Path> {
        let inside = self.texts.range(directory.to_path_buf()..).map(|(path, _)| path.as_path());
        inside.take_while(move |path| path.starts_with(directory))
    }

    pub(crate) fn manifests(&self) -> impl Iterator<Item = &Path> {
        self.texts.keys().map(PathBuf::as_path).filter(|path| path.file_name() == Some(OsStr::new(Tree::MANIFEST)))
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.canonical
    }

    #[must_use]
    pub fn shown(&self, real: &Path) -> Arc<Path> {
        let inside = real.strip_prefix(&self.canonical).ignoring(Ignored::Outside);
        let shown = inside.map_or_else(
            || real.to_path_buf(),
            |relative| {
                if self.given == Path::new(Tree::CURRENT) { relative.to_path_buf() } else { self.given.join(relative) }
            },
        );
        Arc::from(shown)
    }

    pub(crate) fn empty(&self) -> Failure {
        Failure::NoSources { path: self.given.clone() }
    }

    pub(crate) fn path_attribute(attributes: &[Attribute]) -> Option<String> {
        let named = attributes.iter().filter(|attribute| attribute.path().is_ident(Tree::PATH_ATTRIBUTE));
        named.into_iter().find_map(|attribute| match &attribute.meta {
            Meta::NameValue(MetaNameValue { value: Expr::Lit(ExprLit { lit: Lit::Str(path), .. }), .. }) => {
                Some(path.value())
            }
            Meta::NameValue(_) | Meta::Path(_) | Meta::List(_) => None,
        })
    }

    pub(crate) fn lexical(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |folded, component| match component {
            Component::ParentDir => match folded.file_name() {
                Some(_) => folded.parent().map_or_default(Path::to_path_buf),
                None => folded.join(component),
            },
            Component::CurDir => folded,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => folded.join(component),
        })
    }
}
