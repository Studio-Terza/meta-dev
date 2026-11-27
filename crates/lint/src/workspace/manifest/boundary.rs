use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use grammar::Toggled;
use toml::de::DeValue;

use super::{Enabled, Enabling, Manifest, Members};
use crate::terms::Name;

impl Manifest {
    pub(super) const FEATURES: &'static str = "features";
    pub(super) const DEFAULT: &'static str = "default";

    pub(super) fn declared_features(table: Option<&toml::Spanned<DeValue<'_>>>) -> BTreeMap<Name, Vec<Enabling>> {
        let declared = table.and_then(|value| value.get_ref().as_table());
        declared.map_or_default(|features| {
            features
                .iter()
                .map(|(feature, entries)| {
                    let listed = entries.get_ref().as_array().map_or_default(|values| {
                        values.iter().filter_map(|value| value.get_ref().as_str()).map(Enabling::read).collect()
                    });
                    (Name::spelled(feature.get_ref()), listed)
                })
                .collect()
        })
    }

    pub(super) fn requested<'table>(
        directory: &Path,
        sections: impl Iterator<Item = &'table toml::Spanned<DeValue<'table>>>,
    ) -> Vec<Enabled> {
        sections
            .filter_map(|section| section.get_ref().as_table())
            .flat_map(|dependencies| dependencies.iter().map(|(_, dependency)| dependency.get_ref()))
            .filter_map(DeValue::as_table)
            .filter_map(|dependency| {
                let path = dependency.get("path").and_then(|path| path.get_ref().as_str())?;
                let package = Manifest::lexical(&directory.join(path));
                let features = dependency.get(Manifest::FEATURES).and_then(|listed| listed.get_ref().as_array());
                Some(features.map_or_default(|names| {
                    names
                        .iter()
                        .filter_map(|name| name.get_ref().as_str())
                        .map(|feature| Enabled { package: package.clone(), feature: Name::spelled(feature) })
                        .collect::<Vec<Enabled>>()
                }))
            })
            .flatten()
            .collect()
    }

    pub(super) fn named<'table>(
        directory: &Path,
        sections: impl Iterator<Item = &'table toml::Spanned<DeValue<'table>>>,
    ) -> BTreeMap<Name, PathBuf> {
        sections
            .filter_map(|section| section.get_ref().as_table())
            .flat_map(|dependencies| dependencies.iter())
            .filter_map(|(name, dependency)| {
                let path = dependency.get_ref().as_table()?.get("path")?.get_ref().as_str()?;
                Some((Name::spelled(name.get_ref()), Manifest::lexical(&directory.join(path))))
            })
            .collect()
    }
}

impl Enabling {
    fn read(entry: &str) -> Enabling {
        match Toggled::read(entry) {
            Some(Toggled::Own(feature)) => Enabling::Own(Name::spelled(feature)),
            Some(Toggled::Through { dependency, feature }) => {
                Enabling::Dependency { dependency: Name::spelled(dependency), feature: Name::spelled(feature) }
            }
            Some(Toggled::Optional(_)) | None => Enabling::Optional,
        }
    }
}

impl Members {
    const ANY: char = '*';

    pub(super) fn fitting(expected: &Component<'_>, part: &Component<'_>) -> Option<()> {
        let letters: Vec<char> = expected.as_os_str().to_string_lossy().chars().collect();
        let spelled: Vec<char> = part.as_os_str().to_string_lossy().chars().collect();
        Members::fits(&letters, &spelled)
    }

    fn fits(pattern: &[char], text: &[char]) -> Option<()> {
        match pattern.split_first() {
            None => text.is_empty().then_some(()),
            Some((&Members::ANY, rest)) => Members::fits(rest, text).or_else(|| Members::fits(pattern, text.get(1..)?)),
            Some((letter, rest)) => {
                let (first, tail) = text.split_first()?;
                (first == letter).then_some(()).and_then(|()| Members::fits(rest, tail))
            }
        }
    }
}
