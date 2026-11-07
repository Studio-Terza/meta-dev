use lint_core::Site;
use lint_layout::Name;

#[derive(Debug)]
pub struct Declared<'file> {
    method: Name,
    location: Site<'file>,
}

impl<'file> Declared<'file> {
    pub fn new(method: Name, location: Site<'file>) -> Self {
        Declared { method, location }
    }

    pub fn method(&self) -> &Name {
        &self.method
    }

    pub fn location(&self) -> Site<'file> {
        self.location
    }
}
