use lint_layout::Name;

use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Free,
    Inherent,
    TraitDecl,
    TraitImpl,
    Nested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Test,
    Main,
    Called,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Implicit,
    Written(TypeRef),
}

impl Output {
    #[must_use]
    pub fn written(&self) -> Option<&TypeRef> {
        match self {
            Output::Written(ty) => Some(ty),
            Output::Implicit => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    output: Output,
}

impl From<Output> for Signature {
    fn from(output: Output) -> Signature {
        Signature { output }
    }
}

impl Signature {
    #[must_use]
    pub fn output(&self) -> &Output {
        &self.output
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    name: Name,
    owner: Owner,
    entry: Entry,
    signature: Signature,
}

impl Function {
    #[must_use]
    pub fn new(name: Name, owner: Owner, entry: Entry, signature: Signature) -> Function {
        Function { name, owner, entry, signature }
    }

    #[must_use]
    pub fn signed_here(&self) -> Option<&Signature> {
        match self.owner {
            Owner::Free | Owner::Inherent | Owner::TraitDecl | Owner::Nested => Some(&self.signature),
            Owner::TraitImpl => None,
        }
    }

    #[must_use]
    pub fn called(&self) -> Option<&Function> {
        match self.entry {
            Entry::Called => Some(self),
            Entry::Test | Entry::Main => None,
        }
    }

    #[must_use]
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl AsRef<Name> for Function {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
