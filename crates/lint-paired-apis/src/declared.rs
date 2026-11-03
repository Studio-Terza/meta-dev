use crate::owner::Owner;
use lint_core::Site;
use lint_layout::Name;

#[derive(Debug)]
pub struct Declared<'file> {
    owner: Owner,
    method: Name,
    location: Site<'file>,
}

impl<'file> Declared<'file> {
    pub fn new(owner: Owner, method: Name, location: Site<'file>) -> Declared<'file> {
        Declared { owner, method, location }
    }

    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn method(&self) -> &Name {
        &self.method
    }

    pub fn location(&self) -> Site<'file> {
        self.location
    }
}
