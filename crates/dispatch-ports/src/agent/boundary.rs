use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Agent {
    program: String,
    arguments: Vec<String>,
}

impl Agent {
    const CLAUDE: &'static str = "claude";
    const HEADLESS: [&'static str; 8] =
        ["-p", "--output-format", "json", "--setting-sources", "user", "--strict-mcp-config", "--tools", ""];

    #[must_use]
    pub fn claude() -> Agent {
        Agent::new(Agent::CLAUDE, &Agent::HEADLESS)
    }

    #[must_use]
    pub fn new(program: &str, arguments: &[&str]) -> Agent {
        Agent { program: program.to_owned(), arguments: arguments.iter().map(|word| (*word).to_owned()).collect() }
    }

    pub(crate) fn program(&self) -> &str {
        &self.program
    }

    pub(crate) fn arguments(&self) -> &[String] {
        &self.arguments
    }
}
