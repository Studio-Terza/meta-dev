use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use console_ports::{Complaint, Shown};
use file_ports::{Disk, Files, Memory};
use gate_limits::Cli;
use serde::Deserialize;
use test_failure::Failed;

#[test]
fn every_run_prints_its_golden_output() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let bytes = std::fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    let reports = Vec::<Cli>::deserialize(&mut rows)?;
    let past = Vec::<Cli>::deserialize(&mut rows)?;
    let lowered = Vec::<(PathBuf, PathBuf)>::deserialize(&mut rows)?;
    let golden = PathBuf::deserialize(&mut rows)?;
    rows.end()?;
    let reported: Vec<(bool, String, String)> = reports
        .iter()
        .map(|cli| {
            let answered = cli.report_to(&Disk);
            let out: &Shown = answered.as_ref();
            let err: &Complaint = answered.as_ref();
            (answered.code() == ExitCode::SUCCESS, out.to_string(), AsRef::<str>::as_ref(err).to_owned())
        })
        .collect();
    let listed: Vec<Result<Vec<String>, String>> = past
        .iter()
        .map(|cli| match cli.past_warn(&Disk) {
            Ok(findings) => Ok(findings.iter().map(ToString::to_string).collect()),
            Err(failure) => Err(failure.to_string()),
        })
        .collect();
    let written = lowered
        .iter()
        .map(|(limits, measured)| {
            let files: Memory = [limits, measured]
                .into_iter()
                .map(|path| Disk.read(path).map(|held| (path.clone(), held)))
                .collect::<Result<Memory, _>>()?;
            let cli = Cli::from_args([limits, measured].into_iter().map(OsString::from));
            let result = match cli.lower(&files) {
                Ok(record) => Ok(record.to_string()),
                Err(failure) => Err(failure.to_string()),
            };
            Ok((result, files.read_text(limits)?.to_string()))
        })
        .collect::<Result<Vec<(Result<String, String>, String)>, Failed>>()?;
    assert_eq!((reported, listed, written), serde_json::from_slice(&std::fs::read(golden)?)?);
    Ok(())
}
