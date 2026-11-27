use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use console_ports::{Answer, Complaint, Environment, Process, Shown};
use fixture::Properties;
use proptest::prelude::{any, prop_assert_eq, proptest};

#[test]
fn the_process_answers_with_its_arguments_and_the_answer_s_code() {
    let given: Vec<OsString> = env::args_os().collect();
    let expected = given.split_first().map_or_default(|(_, rest)| rest.to_vec());
    let code = Process::tell(|arguments| Answer::new(ExitCode::from(u8::from(arguments != expected))));
    assert_eq!(code, ExitCode::SUCCESS);
    assert_eq!(Process::tell(|_| Answer::new(ExitCode::FAILURE)), ExitCode::FAILURE);
}

#[test]
fn the_process_reads_the_variables_it_was_started_with() {
    assert!(env::vars_os().all(|(name, value)| Process.variable(&name) == Some(value)));
}

proptest! {
    #![proptest_config(Properties::settled())]

    #[test]
    fn an_answer_gathers_what_it_shows_and_keeps_its_complaint(complaint in any::<String>(), first in any::<String>(), second in any::<String>()) {
        let answer = Answer::answered(Err::<Answer, &str>(&complaint)).showing(&first).showing(&second);
        prop_assert_eq!(AsRef::<str>::as_ref(AsRef::<Shown>::as_ref(&answer)), [first, second].concat());
        let complained = AsRef::<str>::as_ref(AsRef::<Complaint>::as_ref(&answer));
        let letters: Vec<char> = complained.chars().collect();
        let ended = letters.split_last().filter(|(end, _)| end.is_ascii_control());
        prop_assert_eq!(ended.map(|(_, said)| said.iter().collect::<String>()), Some(complaint));
        prop_assert_eq!(answer.code(), ExitCode::FAILURE);
    }
}
