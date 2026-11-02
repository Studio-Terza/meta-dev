use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct Place(&'static [&'static str]);

impl Place {
    pub(crate) const GOLDENS: Place = Place(&["goldens"]);
    pub(crate) const SCENARIOS: Place = Place(&["fixtures", "e2e"]);

    const TESTS: &'static str = "tests";

    pub(crate) fn containing(self, path: &Path) -> Option<&Path> {
        path.ancestors().skip(1).find(|ancestor| self.is_beside_tests(ancestor))
    }

    fn is_beside_tests(self, directory: &Path) -> bool {
        directory.file_name().is_some_and(|name| self.0.iter().any(|place| name == *place))
            && directory.parent().and_then(Path::file_name).is_some_and(|parent| parent == Self::TESTS)
    }
}
