use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

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

#[derive(Clone, PartialEq, Eq)]
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

    pub fn written_into(&self, root: &Path) -> io::Result<()> {
        let file = root.join(self.role.file());
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        if self.role == Role::Manifest {
            fs::write(root.join("Cargo.toml"), &self.bytes)?;
            fs::write(file, "")
        } else {
            fs::write(root.join("Cargo.toml"), Role::MANIFEST_TEXT)?;
            fs::write(file, &self.bytes)
        }
    }
}

impl fmt::Debug for Input {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {:?}", self.name(), self.bytes)
    }
}
