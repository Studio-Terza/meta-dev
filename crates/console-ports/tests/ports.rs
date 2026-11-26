use std::ffi::{OsStr, OsString};

use std::process::ExitCode;

use console_ports::{Answer, Complaint, Environment, Process, Shown, Variables};

#[test]
fn the_process_answers_with_its_arguments_and_the_answer_s_code() {
    let expected = std::env::args_os().skip(1).collect::<Vec<OsString>>();
    let code = Process::tell(|arguments| Answer::new(ExitCode::from(u8::from(arguments != expected))));
    assert_eq!(code, ExitCode::SUCCESS);
    assert_eq!(Process::tell(|_| Answer::new(ExitCode::FAILURE)), ExitCode::FAILURE);
}

#[test]
fn an_answer_gathers_what_it_shows_and_keeps_its_complaint() {
    let answer = Answer::complained(ExitCode::FAILURE, "c").showing("a").showing("b");
    assert_eq!(AsRef::<str>::as_ref(AsRef::<Shown>::as_ref(&answer)), "ab");
    assert_eq!(AsRef::<str>::as_ref(AsRef::<Complaint>::as_ref(&answer)), "c");
    assert_eq!(answer.code(), ExitCode::FAILURE);
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
