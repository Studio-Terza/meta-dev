use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, Disk, Files, Memory};
use lint::{Cli, Verdict};
use serde::Deserialize;
use serde::de::IgnoredAny;
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
    let bytes = std::fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    let cases = Vec::<(PathBuf, Verdict, PathBuf, Cli)>::deserialize(&mut rows)?;
    let drifted: Vec<PathBuf> = cases
        .into_iter()
        .map(|(golden, expected, fixture, cli)| {
            let files = Memory::default();
            Disk.mirror(&fixture, &files)?;
            let mut out = Vec::new();
            let mut err = Vec::new();
            let verdict = cli.verdict(&files, Monotonic::default(), &mut out, &mut err);
            let letters: Vec<char> = String::from_utf8(out)?.chars().collect();
            let unnumbered: String = letters
                .chunk_by(|letter, _| !letter.is_ascii_control())
                .flat_map(|line| {
                    line.iter().skip_while(|letter| letter.is_ascii_whitespace() || letter.is_ascii_digit())
                })
                .collect();
            let shown = [unnumbered, String::from_utf8(err)?].concat();
            Ok((verdict != expected || std::fs::read_to_string(&golden)? != shown).then_some(golden))
        })
        .collect::<Result<Vec<Option<PathBuf>>, Failed>>()?
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(drifted, Vec::<PathBuf>::new());
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
    let bytes = std::fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    let (source_at, source_from, manifest_at, manifest_from, unpackaged, packaged) =
        <(PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf)>::deserialize(&mut rows)?;
    let files: Memory = [(source_at, Disk.read(&source_from)?)].into_iter().collect();
    let mut unread = Vec::new();
    let refused = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut unread);
    files.write(&manifest_at, Disk.read(&manifest_from)?)?;
    let mut read = Vec::new();
    let violated = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut read);
    let expected = [Disk.read(&unpackaged)?, Disk.read(&packaged)?];
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
    let bytes = std::fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (fixture, cli, golden) = <(PathBuf, Cli, PathBuf)>::deserialize(&mut rows)?;
    let files = Memory::default();
    Disk.mirror(&fixture, &files)?;
    let mut err = Vec::new();
    let verdict = cli.verdict(&files, Monotonic::default(), &mut Vec::new(), &mut err);
    assert_eq!((verdict, Bytes::new(err)), (Verdict::Clean, Disk.read(&golden)?));
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
    let bytes = std::fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    IgnoredAny::deserialize(&mut rows)?;
    let (workspace, cli, golden) = <(PathBuf, Cli, PathBuf)>::deserialize(&mut rows)?;
    let files = Memory::default();
    Disk.mirror(&workspace, &files)?;
    let mut err = Vec::new();
    let mut nowhere: &mut [u8] = &mut [];
    let verdict = cli.verdict(&files, Monotonic::default(), &mut nowhere, &mut err);
    assert_eq!((verdict, Bytes::new(err)), (Verdict::Refused, Disk.read(&golden)?));
    Ok(())
}
