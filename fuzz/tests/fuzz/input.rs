#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Library,
    Binary,
    Test,
    Manifest,
}

impl Role {
    pub const SOURCES: [Self; 3] = [Self::Library, Self::Binary, Self::Test];

    const MANIFEST_TEXT: &'static str = "[package]\nname = \"fuzzed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

    const fn file(self) -> &'static str {
        match self {
            Self::Library | Self::Manifest => "src/lib.rs",
            Self::Binary => "src/main.rs",
            Self::Test => "tests/fuzzed.rs",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    role: Role,
    bytes: Vec<u8>,
}

impl Input {
    pub const fn new(role: Role, bytes: Vec<u8>) -> Self {
        Self { role, bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn name(&self) -> &'static str {
        match self.role {
            Role::Library => "library",
            Role::Binary => "binary",
            Role::Test => "test",
            Role::Manifest => "manifest",
        }
    }

    pub const fn file(&self) -> &'static str {
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
