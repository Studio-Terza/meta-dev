mod part;

use std::path::{Component, Path};

use lint_core::RepoFile;
use part::Part;

pub(super) struct Platform;

impl Platform {
    pub(super) const NAME: &'static str = "platform";

    pub(super) fn covers_tests(file: &RepoFile) -> bool {
        file.lies_under(Path::new(Self::NAME))
            && file.beneath_named(Path::new(Self::NAME)).is_some_and(|below| Self::names_tests_in(&below))
    }

    fn names_tests_in(below: &Path) -> bool {
        below.components().any(Self::component_names_tests)
    }

    fn component_names_tests(part: Component<'_>) -> bool {
        part.as_os_str().to_string_lossy().parse::<Part>().is_ok_and(|part| part.names_tests())
    }
}
