use lint_core::Site;
use lint_layout::Name;

#[derive(Debug)]
pub struct Declared<'file> {
    method: Name,
    location: Site<'file>,
}

impl<'file> Declared<'file> {
    pub const fn new(method: Name, location: Site<'file>) -> Self {
        Declared { method, location }
    }

    pub const fn method(&self) -> &Name {
        &self.method
    }

    pub const fn location(&self) -> Site<'file> {
        self.location
    }
}
