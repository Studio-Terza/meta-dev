mod target;

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::package_id::PackageId;
pub use target::Form;
use target::Target;

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    id: Option<PackageId>,
    name: CrateName,
    #[serde(default)]
    targets: Vec<Target>,
}

impl Package {
    pub fn package_id(&self) -> Option<&PackageId> {
        self.id.as_ref()
    }

    pub fn library(&self) -> Option<&Target> {
        self.targets.iter().find(|target| target.form().is_some())
    }
}

impl AsRef<CrateName> for Package {
    fn as_ref(&self) -> &CrateName {
        &self.name
    }
}
