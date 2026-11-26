mod package;

use std::path::{Path, PathBuf};

use super::target::Target;
use algebra::Semigroup;

use crate::survey::{Fact, Failed, Survey};
use crate::tree::Tree;
use file_ports::{Files, Kind};
use lint_layout::Failure;
use lint_layout::Manifest;
use newtype::Wrapping;
use package::Package;

#[derive(Debug)]
pub struct Packages;

impl Packages {
    pub fn targets_of(
        files: &impl Files,
        trees: &[&Tree<'_>],
        admitted: Vec<Result<Vec<Manifest>, Failure>>,
    ) -> Vec<Result<Vec<Target>, Failure>> {
        let named_paths: Vec<PathBuf> = admitted.iter().flatten().flatten().flat_map(Manifest::named_paths).collect();
        let named = Survey::learned(files, Fact::Canonical, &named_paths);
        let groups: Vec<Result<Vec<Package>, Failure>> =
            admitted.into_iter().map(|group| group.map(|manifests| Self::packages(&named, &manifests))).collect();
        let packages: Vec<&Package> = groups.iter().flatten().flatten().collect();
        Self::surveyed(files, &packages).settling(|known, failed| Self::targeted(trees, groups, known, failed))
    }

    fn targeted(
        trees: &[&Tree<'_>],
        groups: Vec<Result<Vec<Package>, Failure>>,
        known: &Survey,
        failed: Failed,
    ) -> Vec<Result<Vec<Target>, Failure>> {
        let real: Vec<Vec<PathBuf>> = groups.iter().map(|group| Self::real(known, group)).collect();
        let orders: Vec<Vec<&Path>> = real.iter().map(|order| order.iter().map(PathBuf::as_path).collect()).collect();
        let outcomes = failed.settled(known, &orders);
        trees
            .iter()
            .zip(groups)
            .zip(outcomes)
            .map(|((tree, group), outcome)| {
                let members = group?;
                let settled = outcome?;
                Ok(members.into_iter().filter_map(|package| package.into_target(settled, tree)).collect())
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
        let directories: Vec<PathBuf> =
            packages.iter().flat_map(|package| package.target_directories()).map(Wrapping::into_inner).collect();
        let followed = Survey::learned(files, Fact::Followed, &candidates).combine(Survey::learned(
            files,
            Fact::Kind,
            &directories,
        ));
        let real: Vec<PathBuf> =
            directories.into_iter().filter(|directory| followed.kind(directory) == Kind::Directory).collect();
        let listed = followed.combine(Survey::learned(files, Fact::Listed, &real));
        let entries: Vec<PathBuf> = real.iter().flat_map(|directory| listed.entries(directory)).cloned().collect();
        let survey = listed.combine(Survey::learned(files, Fact::Kind, &entries));
        let mains: Vec<PathBuf> = entries
            .iter()
            .filter(|entry| survey.kind(entry) == Kind::Directory)
            .map(|entry| entry.join(Package::BINARY_ROOT))
            .collect();
        survey.combine(Survey::learned(files, Fact::Followed, &mains))
    }

    fn real(known: &Survey, group: &Result<Vec<Package>, Failure>) -> Vec<PathBuf> {
        group
            .iter()
            .flatten()
            .flat_map(Package::target_directories)
            .map(Wrapping::into_inner)
            .filter(|directory| known.kind(directory) == Kind::Directory)
            .collect()
    }
}
