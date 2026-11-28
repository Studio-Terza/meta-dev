use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use file_ports::{Files, Kind, Text};
use invariants::Ignoring;
use syn::{Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue};

use crate::failure::Failure;
use crate::ignored::Ignored;

#[derive(Debug)]
pub(crate) struct Tree {
    given: PathBuf,
    canonical: PathBuf,
    texts: BTreeMap<PathBuf, Text>,
}

impl Tree {
    const CURRENT: &'static str = ".";
    pub(crate) const MANIFEST: &'static str = "Cargo.toml";
    const BUILD_OUTPUT: &'static str = "target";
    const HIDDEN: u8 = b'.';
    const SOURCE_EXTENSION: &'static str = "rs";
    const PATH_ATTRIBUTE: &'static str = "path";

    pub(crate) fn read(files: &impl Files, working: &Path, given: &Path) -> Result<Tree, Failure> {
        let canonical = files
            .canonical(&working.join(given))
            .map_err(|source| Failure::Read { path: given.to_path_buf(), source })?;
        let above: BTreeMap<PathBuf, Text> = canonical
            .ancestors()
            .skip(1)
            .filter_map(|directory| {
                let manifest = directory.join(Tree::MANIFEST);
                files.read_text(&manifest).ignoring(Ignored::Unlisted).map(|text| (manifest, text))
            })
            .collect();
        let outline = Tree { given: given.to_path_buf(), canonical, texts: above };
        let below = Tree::listed(files, &outline.canonical).into_iter().map(|path| match files.read_text(&path) {
            Ok(text) => Ok((path, text)),
            Err(source) => Err(Failure::Read { path: outline.shown(&path).to_path_buf(), source }),
        });
        let read = below.collect::<Result<Vec<(PathBuf, Text)>, Failure>>()?;
        Ok(Tree { texts: outline.texts.into_iter().chain(read).collect(), ..outline })
    }

    fn listed(files: &impl Files, directory: &Path) -> Vec<PathBuf> {
        let manifested = files.kind(&directory.join(Tree::MANIFEST));
        let entries = files.entries(directory).ignoring(Ignored::Unlisted).unwrap_or_default();
        entries
            .into_iter()
            .flat_map(|entry| {
                let name = entry.file_name().unwrap_or_default();
                let hidden = name.as_encoded_bytes().first() == Some(&Tree::HIDDEN);
                let built = manifested == Kind::File && name == Tree::BUILD_OUTPUT;
                let source = name == Tree::MANIFEST || entry.extension() == Some(OsStr::new(Tree::SOURCE_EXTENSION));
                match files.kind(&entry) {
                    Kind::Directory if !hidden && !built => Tree::listed(files, &entry),
                    Kind::File if source => vec![entry],
                    Kind::Directory | Kind::File | Kind::Link | Kind::Missing => Vec::new(),
                }
            })
            .collect()
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

    pub(crate) fn root(&self) -> &Path {
        &self.canonical
    }

    pub(crate) fn shown(&self, real: &Path) -> Arc<Path> {
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
