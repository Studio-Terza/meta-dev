use crate::owner::Owner;
use lint_core::Site;
use lint_layout::Name;

#[derive(Debug)]
pub struct Declared {
    owner: Owner,
    method: Name,
    location: Site,
}

impl Declared {
    pub fn new(owner: Owner, method: Name, location: Site) -> Declared {
        Declared { owner, method, location }
    }

    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn method(&self) -> &Name {
        &self.method
    }

    pub fn location(&self) -> &Site {
        &self.location
    }
}
