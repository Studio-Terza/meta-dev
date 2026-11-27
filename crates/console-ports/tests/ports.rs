use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use console_ports::{Answer, Complaint, Environment, Heard, Process, Shown, Variables};
use fixture::Properties;
use proptest::prelude::{any, prop_assert, prop_assert_eq, prop_assume, proptest};

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

    #[test]
    fn in_memory_variables_answer_from_their_pairs_and_hear_what_they_were_given(
        pairs in any::<Vec<(String, String)>>(),
        unnamed in any::<String>(),
        heard in any::<Heard>(),
    ) {
        let named: BTreeMap<OsString, OsString> =
            pairs.into_iter().map(|(name, value)| (OsString::from(name), OsString::from(value))).collect();
        let absent = OsString::from(unnamed);
        prop_assume!(!named.contains_key(&absent));
        let variables = named.clone().into_iter().collect::<Variables>().hearing(heard.clone());
        prop_assert_eq!(variables.variable(&absent), None);
        prop_assert!(named.iter().all(|(name, value)| variables.variable(name).as_ref() == Some(value)));
        prop_assert_eq!(variables.input()?, heard);
    }
}
