mod package;

use std::path::{Path, PathBuf};

use super::target::Target;
use crate::survey::{Fact, Survey};
use crate::tree::Tree;
use file_ports::{Files, Kind};
use lint_layout::Failure;
use lint_layout::Manifest;
use package::Package;

#[derive(Debug)]
pub struct Packages;

impl Packages {
    pub fn targets_of(
        files: &impl Files,
        trees: &[&Tree],
        admitted: Vec<Result<Vec<Manifest>, Failure>>,
    ) -> Vec<Result<Vec<Target>, Failure>> {
        let named: Vec<PathBuf> = admitted.iter().flatten().flatten().flat_map(Manifest::named_paths).collect();
        let named = Survey::default().learn(files, Fact::Canonical, &named);
        let groups: Vec<Result<Vec<Package>, Failure>> =
            admitted.into_iter().map(|group| group.map(|manifests| Self::packages(&named, &manifests))).collect();
        let packages: Vec<&Package> = groups.iter().flatten().flatten().collect();
        let mut survey = Self::surveyed(files, &packages);
        trees
            .iter()
            .zip(groups)
            .map(|(tree, group)| {
                let packages = group?;
                Self::checked(&mut survey, &packages)?;
                Ok(packages.iter().flat_map(|package| package.targets(&survey, tree)).collect())
            })
            .collect()
    }

    fn packages(named: &Survey, manifests: &[Manifest]) -> Vec<Package> {
        let canonical = |path: &Path| named.canonical(path).map(Path::to_path_buf);
        manifests
            .iter()
            .map(|manifest| {
                Package::in_directory(manifest.directory().to_path_buf(), manifest.declared_roots(canonical))
            })
            .collect()
    }

    fn surveyed(files: &impl Files, packages: &[&Package]) -> Survey {
        let candidates: Vec<PathBuf> = packages.iter().flat_map(|package| package.candidates()).collect();
        let directories: Vec<PathBuf> = packages.iter().flat_map(|package| package.target_directories()).collect();
        let survey = Survey::default().learn(files, Fact::Followed, &candidates).learn(files, Fact::Kind, &directories);
        let real: Vec<PathBuf> =
            directories.into_iter().filter(|directory| survey.kind(directory) == Kind::Directory).collect();
        let survey = survey.learn(files, Fact::Listed, &real);
        let entries: Vec<PathBuf> = real.iter().flat_map(|directory| survey.entries(directory)).cloned().collect();
        let survey = survey.learn(files, Fact::Kind, &entries);
        let mains: Vec<PathBuf> = entries
            .iter()
            .filter(|entry| survey.kind(entry) == Kind::Directory)
            .map(|entry| entry.join(Package::BINARY_ROOT))
            .collect();
        survey.learn(files, Fact::Followed, &mains)
    }

    fn checked(survey: &mut Survey, packages: &[Package]) -> Result<(), Failure> {
        let real: Vec<PathBuf> = packages
            .iter()
            .flat_map(Package::target_directories)
            .filter(|directory| survey.kind(directory) == Kind::Directory)
            .collect();
        survey.failure(&real).map_or(Ok(()), Err)
    }
}
