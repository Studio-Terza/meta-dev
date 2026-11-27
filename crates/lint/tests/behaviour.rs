use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, Disk, Files, Memory};
use lint::{Cli, Verdict};
use test_failure::Failed;
use timer_ports::Monotonic;

const VERDICTS: [Verdict; 3] = [Verdict::Clean, Verdict::Violations, Verdict::Refused];

#[test]
fn every_case_prints_its_golden_output() -> Result<(), Failed> {
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
    let Some([out_marker, err_marker, words @ ..]) = header.first().map(Vec::as_slice) else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let drifted: Vec<String> = cases
        .iter()
        .map(|case| {
            let [golden, fixture, arguments @ ..] = case.as_slice() else {
                return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
            };
            let files = Memory::default();
            Disk.mirror(Path::new(fixture), &files)?;
            let mut out = Vec::new();
            let mut err = Vec::new();
            let given = arguments.iter().map(OsString::from);
            let verdict = Cli::from_args(given).verdict(&files, Monotonic::default(), &mut out, &mut err);
            let letters: Vec<char> = String::from_utf8(out)?.chars().collect();
            let unnumbered: String = letters
                .chunk_by(|letter, _| !letter.is_ascii_control())
                .flat_map(|line| {
                    line.iter().skip_while(|letter| letter.is_ascii_whitespace() || letter.is_ascii_digit())
                })
                .collect();
            let said =
                VERDICTS.iter().zip(words).filter(|(known, _)| **known == verdict).map(|(_, word)| word.as_str());
            let shown: String =
                said.chain([out_marker.as_str(), &unnumbered, err_marker, &String::from_utf8(err)?]).collect();
            Ok((std::fs::read_to_string(golden)? != shown).then(|| golden.clone()))
        })
        .collect::<Result<Vec<Option<String>>, Failed>>()?
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(drifted, Vec::<String>::new());
    Ok(())
}

#[test]
fn no_argument_reads_the_current_directory() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let sections: Vec<Vec<Vec<String>>> = serde_json::from_slice(&std::fs::read(found)?)?;
    let [_, _, current, ..] = sections.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let Some([source_at, source_from, manifest_at, manifest_from, unpackaged, packaged]) =
        current.first().map(Vec::as_slice)
    else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let files: Memory = [(PathBuf::from(source_at), Disk.read(Path::new(source_from))?)].into_iter().collect();
    let mut unread = Vec::new();
    let refused = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut unread);
    files.write(Path::new(manifest_at), Disk.read(Path::new(manifest_from))?)?;
    let mut read = Vec::new();
    let violated = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut read);
    let expected = [Disk.read(Path::new(unpackaged))?, Disk.read(Path::new(packaged))?];
    assert_eq!(
        (refused, violated, [Bytes::new(unread), Bytes::new(read)]),
        (Verdict::Refused, Verdict::Violations, expected)
    );
    Ok(())
}

#[test]
fn verdicts_become_exit_codes() {
    assert_eq!(VERDICTS.map(ExitCode::from), [ExitCode::SUCCESS, ExitCode::FAILURE, ExitCode::FAILURE]);
}

#[test]
fn deeply_nested_code_is_read_whole() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let sections: Vec<Vec<Vec<String>>> = serde_json::from_slice(&std::fs::read(found)?)?;
    let [_, _, _, deep, ..] = sections.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let Some([manifest_at, manifest_from, source_at, head, open, core, close, tail, depth, golden]) =
        deep.first().map(Vec::as_slice)
    else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let nesting: usize = depth.parse()?;
    let nested = [head.clone(), open.repeat(nesting), core.clone(), close.repeat(nesting), tail.clone()].concat();
    let files: Memory = [
        (PathBuf::from(manifest_at), Disk.read(Path::new(manifest_from))?),
        (PathBuf::from(source_at), Bytes::new(nested.into_bytes())),
    ]
    .into_iter()
    .collect();
    let mut err = Vec::new();
    let verdict = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut err);
    assert_eq!((verdict, Bytes::new(err)), (Verdict::Clean, Disk.read(Path::new(golden))?));
    Ok(())
}

#[test]
fn output_that_cannot_be_written_refuses_the_run() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let sections: Vec<Vec<Vec<String>>> = serde_json::from_slice(&std::fs::read(found)?)?;
    let [_, _, _, _, unwritable, ..] = sections.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let Some([workspace, golden, arguments @ ..]) = unwritable.first().map(Vec::as_slice) else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let files = Memory::default();
    Disk.mirror(Path::new(workspace), &files)?;
    let mut err = Vec::new();
    let mut nowhere: &mut [u8] = &mut [];
    let given = arguments.iter().map(OsString::from);
    let verdict = Cli::from_args(given).verdict(&files, Monotonic::default(), &mut nowhere, &mut err);
    assert_eq!((verdict, Bytes::new(err)), (Verdict::Refused, Disk.read(Path::new(golden))?));
    Ok(())
}
