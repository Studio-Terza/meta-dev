#![deny(clippy::wildcard_enum_match_arm)]

mod ports {
    use std::ffi::OsString;
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Stdio};

    use grammar_slug::Letters;
    use tempfile::TempDir;
    use test_failure::Failed;

    const DESIGN: &str = env!("CARGO_BIN_EXE_design");
    const HOME: &str = "HOME";
    const WORK: &str = "work";
    const PROJECTS: &str = ".claude/projects";
    const WORKSPACE: &str = "--workspace";
    const SCENARIOS: &str = "tests/goldens/scenarios.txt";
    const RECORD: &str = "RECORD_GOLDENS";
    const EXIT: &str = "--- exit ";
    const OUT: &str = "\n--- out\n";
    const ERR: &str = "--- err\n";
    const STOP: &[&str] = &["hook", "stop"];
    const CASES: [(&[&str], &str); 3] = [
        (&["frobnicate"], "tests/fixtures/empty.txt"),
        (STOP, "tests/fixtures/stop-active.json"),
        (STOP, "tests/fixtures/not-json.txt"),
    ];

    #[test]
    fn each_scenario_prints_its_golden_output() -> Result<(), Failed> {
        let home = TempDir::new()?;
        let workspace = home.path().join(WORK);
        let slug = Letters::slugged(&workspace.to_string_lossy());
        std::fs::create_dir_all(home.path().join(PROJECTS).join(slug))?;
        let shown = CASES
            .iter()
            .map(|(words, input)| {
                let arguments = [OsString::from(WORKSPACE), workspace.clone().into_os_string()]
                    .into_iter()
                    .chain(words.iter().map(OsString::from));
                let mut child = Command::new(DESIGN)
                    .env(HOME, home.path())
                    .current_dir(home.path())
                    .args(arguments)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                let fed = std::fs::read(Path::new(input))?;
                child.stdin.take().into_iter().try_for_each(|mut stdin| stdin.write_all(&fed))?;
                let output = child.wait_with_output()?;
                let code = output.status.code().map_or_default(|code| code.to_string());
                Ok([
                    EXIT.to_owned(),
                    code,
                    OUT.to_owned(),
                    String::from_utf8(output.stdout)?,
                    ERR.to_owned(),
                    String::from_utf8(output.stderr)?,
                ]
                .concat())
            })
            .collect::<Result<String, Failed>>()?;
        std::env::var_os(RECORD).map_or(Ok(()), |_| std::fs::write(Path::new(SCENARIOS), &shown))?;
        assert_eq!(shown, std::fs::read_to_string(Path::new(SCENARIOS))?);
        Ok(())
    }
}
