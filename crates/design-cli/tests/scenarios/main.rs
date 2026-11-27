#![deny(clippy::wildcard_enum_match_arm)]

mod ports {
    use std::fs;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use console_ports::Variables;
    use design::Arguments;
    use serde::Deserialize;
    use tempfile::Builder;
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
        let bytes = fs::read(found)?;
        let mut rows = serde_json::Deserializer::from_slice(&bytes);
        let binary = PathBuf::deserialize(&mut rows)?;
        let environment = Variables::deserialize(&mut rows)?;
        let home = PathBuf::deserialize(&mut rows)?;
        let projects = PathBuf::deserialize(&mut rows)?;
        let golden = PathBuf::deserialize(&mut rows)?;
        let cases = Vec::<(PathBuf, Arguments)>::deserialize(&mut rows)?;
        let design = std::env::current_exe()?
            .ancestors()
            .map(|directory| directory.join(&binary))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        fs::create_dir_all(&home)?;
        fs::remove_dir_all(&home)?;
        let scratch = Builder::new()
            .prefix(home.file_name().unwrap_or_default())
            .rand_bytes(usize::default())
            .tempdir_in(home.parent().unwrap_or(&home))?;
        fs::create_dir_all(&projects)?;
        let shown = cases
            .into_iter()
            .map(|(input, arguments)| {
                let mut child = Command::new(&design)
                    .envs(environment.iter())
                    .current_dir(&home)
                    .args(arguments)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                let fed = fs::read(input)?;
                child.stdin.take().into_iter().try_for_each(|mut stdin| stdin.write_all(&fed))?;
                let output = child.wait_with_output()?;
                Ok((output.status.code(), String::from_utf8(output.stdout)?, String::from_utf8(output.stderr)?))
            })
            .collect::<Result<Vec<_>, Failed>>()?;
        scratch.close()?;
        assert_eq!(shown, serde_json::from_slice::<Vec<_>>(&fs::read(golden)?)?);
        Ok(())
    }
}
