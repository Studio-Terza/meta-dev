use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Program {
    name: String,
    arguments: Vec<String>,
}

impl Program {
    const CLAUDE: &'static str = "claude";
    const HEADLESS: [&'static str; 8] =
        ["-p", "--output-format", "json", "--setting-sources", "user", "--strict-mcp-config", "--tools", ""];

    #[must_use]
    pub fn new(name: &str, arguments: Vec<String>) -> Program {
        Program { name: name.to_owned(), arguments }
    }

    #[must_use]
    pub fn claude() -> Program {
        Program::new(Program::CLAUDE, Program::HEADLESS.map(str::to_owned).to_vec())
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn arguments(&self) -> &[String] {
        &self.arguments
    }
}
