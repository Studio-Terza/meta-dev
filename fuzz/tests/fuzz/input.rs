#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Library,
    Binary,
    Test,
    Manifest,
}

impl Role {
    pub const SOURCES: [Role; 3] = [Role::Library, Role::Binary, Role::Test];

    const MANIFEST_TEXT: &'static str = "[package]\nname = \"fuzzed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

    fn file(self) -> &'static str {
        match self {
            Role::Library | Role::Manifest => "src/lib.rs",
            Role::Binary => "src/main.rs",
            Role::Test => "tests/fuzzed.rs",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    role: Role,
    bytes: Vec<u8>,
}

impl Input {
    pub fn new(role: Role, bytes: Vec<u8>) -> Input {
        Input { role, bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn name(&self) -> &'static str {
        match self.role {
            Role::Library => "library",
            Role::Binary => "binary",
            Role::Test => "test",
            Role::Manifest => "manifest",
        }
    }

    pub fn file(&self) -> &'static str {
        self.role.file()
    }

    pub fn manifest(&self) -> &[u8] {
        match self.role {
            Role::Manifest => &self.bytes,
            Role::Library | Role::Binary | Role::Test => Role::MANIFEST_TEXT.as_bytes(),
        }
    }

    pub fn source(&self) -> &[u8] {
        match self.role {
            Role::Manifest => &[],
            Role::Library | Role::Binary | Role::Test => &self.bytes,
        }
    }
}
