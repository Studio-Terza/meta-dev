#![deny(clippy::wildcard_enum_match_arm)]

mod ports {
    use std::ffi::OsString;
    use std::io::{self, Write};
    use std::path::Path;
    use std::process::{Command, Stdio};

    use grammar_slug::Letters;
    use tempfile::TempDir;
    use test_failure::Failed;

    #[test]
    fn each_scenario_prints_its_golden_output() -> Result<(), Failed> {
        let source = Path::new(file!());
        let table = source.with_file_name(source.file_stem().unwrap_or_default());
        let found = std::env::current_dir()?
            .ancestors()
            .map(|directory| directory.join(&table))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let sections: Vec<Vec<Vec<String>>> = serde_json::from_slice(&std::fs::read(found)?)?;
        let [header, cases, ..] = sections.as_slice() else {
            return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
        };
        let Some([binary, home_variable, workspace_flag, projects, work, golden, exit, out, err]) =
            header.first().map(Vec::as_slice)
        else {
            return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
        };
        let design = std::env::current_exe()?
            .ancestors()
            .map(|directory| directory.join(binary))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let home = TempDir::new()?;
        let workspace = home.path().join(work);
        let slug = Letters::slugged(&workspace.to_string_lossy());
        std::fs::create_dir_all(home.path().join(projects).join(slug))?;
        let shown = cases
            .iter()
            .map(|case| {
                let [input, words @ ..] = case.as_slice() else {
                    return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
                };
                let arguments = [OsString::from(workspace_flag), workspace.clone().into_os_string()]
                    .into_iter()
                    .chain(words.iter().map(OsString::from));
                let mut child = Command::new(&design)
                    .env(home_variable, home.path())
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
                    exit.clone(),
                    code,
                    out.clone(),
                    String::from_utf8(output.stdout)?,
                    err.clone(),
                    String::from_utf8(output.stderr)?,
                ]
                .concat())
            })
            .collect::<Result<String, Failed>>()?;
        assert_eq!(shown, std::fs::read_to_string(golden)?);
        Ok(())
    }
}
