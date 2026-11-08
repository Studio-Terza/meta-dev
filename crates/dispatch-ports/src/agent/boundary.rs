#[derive(Clone, Copy, Debug)]
pub struct Agent {
    program: &'static str,
    arguments: &'static [&'static str],
}

impl Agent {
    pub const CLAUDE: Agent = Agent {
        program: "claude",
        arguments: &["-p", "--model", "haiku", "--output-format", "json", "--setting-sources", "user", "--tools", ""],
    };

    #[must_use]
    pub const fn new(program: &'static str, arguments: &'static [&'static str]) -> Agent {
        Agent { program, arguments }
    }

    pub(crate) fn program(self) -> &'static str {
        self.program
    }

    pub(crate) fn arguments(self) -> &'static [&'static str] {
        self.arguments
    }
}
