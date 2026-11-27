use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use console_ports::{Complaint, Shown};
use file_ports::{Disk, Files, Memory};
use gate_limits::Cli;
use test_failure::Failed;

const CODES: [ExitCode; 2] = [ExitCode::SUCCESS, ExitCode::FAILURE];

#[test]
fn every_run_prints_its_golden_output() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let sections: Vec<Vec<Vec<String>>> = serde_json::from_slice(&std::fs::read(found)?)?;
    let [header, reports, past, lowered, ..] = sections.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let Some([out_marker, err_marker, written_marker, newline, words @ ..]) = header.first().map(Vec::as_slice) else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let reported = reports.iter().map(|row| {
        let [golden, arguments @ ..] = row.as_slice() else {
            return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
        };
        let answered = Cli::from_args(arguments.iter().map(OsString::from)).report_to(&Disk);
        let code = answered.code();
        let said = CODES.iter().zip(words).filter(|(known, _)| **known == code).map(|(_, word)| word.as_str());
        let out: &Shown = answered.as_ref();
        let err: &Complaint = answered.as_ref();
        let shown: String = said
            .chain([out_marker.as_str(), AsRef::<str>::as_ref(out), err_marker, AsRef::<str>::as_ref(err)])
            .collect();
        Ok((golden.clone(), shown))
    });
    let listed = past.iter().map(|row| {
        let [golden, arguments @ ..] = row.as_slice() else {
            return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
        };
        let cli = Cli::from_args(arguments.iter().map(OsString::from));
        let shown = match cli.past_warn(&Disk) {
            Ok(findings) => findings.iter().map(|finding| [finding.to_string(), newline.clone()].concat()).collect(),
            Err(failure) => [failure.to_string(), newline.clone()].concat(),
        };
        Ok((golden.clone(), shown))
    });
    let lowered_runs = lowered.iter().map(|row| {
        let [golden, limits, measured] = row.as_slice() else {
            return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
        };
        let files: Memory = [limits, measured]
            .into_iter()
            .map(|path| Disk.read(Path::new(path)).map(|bytes| (Path::new(path).to_path_buf(), bytes)))
            .collect::<Result<Memory, _>>()?;
        let cli = Cli::from_args([limits, measured].into_iter().map(OsString::from));
        let result = match cli.lower(&files) {
            Ok(record) => record.to_string(),
            Err(failure) => [failure.to_string(), newline.clone()].concat(),
        };
        let shown = [result, written_marker.clone(), files.read_text(Path::new(limits))?.to_string()].concat();
        Ok((golden.clone(), shown))
    });
    let drifted: Vec<String> = reported
        .chain(listed)
        .chain(lowered_runs)
        .map(|run| run.and_then(|(golden, shown)| Ok((std::fs::read_to_string(&golden)? != shown).then_some(golden))))
        .collect::<Result<Vec<Option<String>>, Failed>>()?
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(drifted, Vec::<String>::new());
    Ok(())
}
