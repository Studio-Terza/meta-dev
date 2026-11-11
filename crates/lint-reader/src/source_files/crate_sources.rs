use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cargo_ports::Cargo;
use file_ports::Files;
use lint_layout::{CrateSource, Crates, Dependencies, LibraryForm, Manifest, Name, PackageName, SourceText};
use newtype::Wrapping;
use workspace_metadata::{CrateName, Form, Library, Metadata};

use super::{Described, Unlocated};

#[derive(Debug)]
pub struct CrateSources;

impl CrateSources {
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn read(files: &impl Files, cargo: &impl Cargo, packages: &[&Path]) -> Result<Described, Unlocated> {
        let manifests: Vec<PathBuf> =
            Self::workspaces(files, packages).iter().map(|workspace| workspace.join(Self::MANIFEST)).collect();
        let read: Vec<Metadata> = manifests
            .iter()
            .zip(cargo.metadata(&manifests))
            .map(|(manifest, answer)| {
                serde_json::from_slice::<Metadata>(answer?.as_ref())
                    .map_err(|source| Unlocated::Unparsed { manifest: manifest.clone(), source })
            })
            .collect::<Result<_, _>>()?;
        Self::crates(files, &read).map(|crates| Described::new(crates, read))
    }

    fn crates(files: &impl Files, read: &[Metadata]) -> Result<Crates, Unlocated> {
        let libraries: Vec<Library<'_>> = read.iter().flat_map(Metadata::libraries).collect();
        let roots: Vec<&Path> = libraries.iter().map(Library::root).collect();
        libraries
            .iter()
            .zip(roots.iter().zip(files.read_text_many(&roots)))
            .map(|(library, (root, text))| {
                let dependencies: Dependencies = library
                    .dependencies()
                    .iter()
                    .map(|(krate, package)| (Name::from(krate.as_ref()), Self::package(package)))
                    .collect();
                let found = text.map_err(|source| Unlocated::Root { root: root.to_path_buf(), source })?;
                let source =
                    CrateSource::new(SourceText::new(found.into_inner()), Self::form(library.form()), dependencies);
                Ok((Self::package(library.name()), source))
            })
            .collect()
    }

    fn form(form: Form) -> LibraryForm {
        match form {
            Form::Library => LibraryForm::Rust,
            Form::ProcMacro => LibraryForm::ProcMacro,
        }
    }

    fn package(name: &CrateName) -> PackageName {
        PackageName::from(&Name::from(name.as_ref()))
    }

    fn workspaces(files: &impl Files, packages: &[&Path]) -> BTreeSet<PathBuf> {
        let candidates: Vec<&Path> =
            packages.iter().flat_map(|package| package.ancestors()).collect::<BTreeSet<&Path>>().into_iter().collect();
        let enclosing: BTreeSet<&Path> = candidates
            .iter()
            .zip(Manifest::read_many(files, &candidates))
            .filter_map(|(directory, read)| read.ok().flatten()?.workspace().map(|_| *directory))
            .collect();
        packages
            .iter()
            .map(|package| {
                package.ancestors().find(|directory| enclosing.contains(directory)).unwrap_or(package).to_path_buf()
            })
            .collect()
    }
}
