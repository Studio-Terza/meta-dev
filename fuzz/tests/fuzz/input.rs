use strum::IntoStaticStr;

#[derive(IntoStaticStr, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Role {
    Library,
    Binary,
    Test,
    Manifest,
}

#[derive(IntoStaticStr, Clone, Copy, Debug, PartialEq, Eq)]
enum File {
    #[strum(serialize = "src/lib.rs")]
    Library,
    #[strum(serialize = "src/main.rs")]
    Binary,
    #[strum(serialize = "tests/fuzzed.rs")]
    Test,
}

impl Role {
    pub const SOURCES: [Role; 3] = [Role::Library, Role::Binary, Role::Test];

    const MANIFEST_TEXT: &'static str = "[package]\nname = \"fuzzed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

    fn file(self) -> File {
        match self {
            Role::Library | Role::Manifest => File::Library,
            Role::Binary => File::Binary,
            Role::Test => File::Test,
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
        self.role.into()
    }

    pub fn file(&self) -> &'static str {
        self.role.file().into()
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
