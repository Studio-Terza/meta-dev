mod part;

use std::path::{Component, Path};

use lint_core::RepoFile;
use part::Part;

pub(super) struct Platform;

impl Platform {
    pub(super) const NAME: &'static str = "platform";

    pub(super) fn tests_covering(file: &RepoFile) -> Option<&RepoFile> {
        file.lies_under(Path::new(Self::NAME))
            .and_then(|file| file.beneath_named(Path::new(Self::NAME)))
            .and_then(|below| Self::tests_named_in(&below))
            .map(|_| file)
    }

    fn tests_named_in(below: &Path) -> Option<Part> {
        below.components().find_map(Self::component_naming_tests)
    }

    fn component_naming_tests(part: Component<'_>) -> Option<Part> {
        part.as_os_str().to_string_lossy().parse::<Part>().ok().and_then(Part::naming_tests)
    }
}
