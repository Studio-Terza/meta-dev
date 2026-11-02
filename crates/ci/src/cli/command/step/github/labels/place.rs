use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct Place(&'static [&'static str]);

impl Place {
    pub(crate) const GOLDENS: Place = Place(&["goldens"]);
    pub(crate) const SCENARIOS: Place = Place(&["fixtures", "e2e"]);

    const TESTS: &'static str = "tests";

    pub(crate) fn containing(self, path: &Path) -> Option<&Path> {
        path.ancestors().skip(1).find_map(|ancestor| self.beside_tests(ancestor))
    }

    fn beside_tests(self, directory: &Path) -> Option<&Path> {
        directory
            .file_name()
            .filter(|name| self.0.iter().any(|place| name == place))
            .and(directory.parent().and_then(Path::file_name))
            .filter(|parent| *parent == Self::TESTS)
            .map(|_| directory)
    }
}
