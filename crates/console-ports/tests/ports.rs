use std::ffi::{OsStr, OsString};

use console_ports::{Environment, Process, Variables};

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
