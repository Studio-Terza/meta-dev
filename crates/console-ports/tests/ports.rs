use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use console_ports::{Cargo, Environment, Process, Toolchain, Variables};

#[test]
fn the_process_answers_with_its_arguments() {
    let answered = Process::answer(|arguments, _, _| arguments);
    assert_eq!(answered, std::env::args_os().skip(1).collect::<Vec<OsString>>());
}

#[test]
fn the_process_reads_its_variables() {
    assert!(Process.variable(OsStr::new("PATH")).is_some());
    assert_eq!(Process.variable(OsStr::new("CONSOLE_PORTS_SURELY_UNSET")), None);
}

#[test]
fn fake_variables_answer_from_their_pairs() {
    let variables: Variables = [(OsString::from("CI"), OsString::from("true"))].into_iter().collect();
    assert_eq!(variables.variable(OsStr::new("CI")), Some(OsString::from("true")));
    assert_eq!(variables.variable(OsStr::new("HOME")), None);
}

#[test]
fn the_toolchain_describes_a_workspace_and_refuses_a_directory_without_one() {
    let described = Toolchain.metadata(&[PathBuf::from(env!("CARGO_MANIFEST_DIR")), Path::new("/").to_path_buf()]);
    let lengths: Vec<Option<usize>> =
        described.iter().map(|read| read.as_ref().ok().map(|text| text.as_ref().len())).collect();
    assert_eq!(lengths.len(), 2);
    assert_ne!(lengths.first(), Some(&Some(0)));
    assert_ne!(lengths.first(), Some(&None));
    assert_eq!(lengths.get(1), Some(&None));
}
