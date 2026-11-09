mod target;

use serde::Deserialize;

use target::Target;

#[derive(Debug, Deserialize)]
pub(super) struct Package {
    id: String,
    name: String,
    targets: Vec<Target>,
}

impl Package {
    pub(super) fn among(&self, members: &[String]) -> bool {
        members.contains(&self.id)
    }

    pub(super) fn named(&self, names: &[String]) -> bool {
        names.contains(&self.name)
    }

    pub(super) fn kinds(&self) -> impl Iterator<Item = &str> {
        self.targets.iter().flat_map(Target::kinds)
    }
}
