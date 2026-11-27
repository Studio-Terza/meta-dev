use std::ffi::{OsStr, OsString};
use std::process::ExitCode;

use console_ports::{Answer, Complaint, Environment, Process, Shown, Variables};
use proptest::prelude::{any, prop_assert_eq, proptest};

proptest! {
    #[test]
    fn the_process_answers_with_the_answer_s_code(code in any::<u8>()) {
        prop_assert_eq!(Process::tell(|_| Answer::new(ExitCode::from(code))), ExitCode::from(code));
    }

    #[test]
    fn an_answer_gathers_what_it_shows_and_keeps_its_complaint(
        first in any::<String>(),
        second in any::<String>(),
        complaint in any::<String>(),
        code in any::<u8>(),
    ) {
        let answer = Answer::complained(ExitCode::from(code), &complaint).showing(&first).showing(&second);
        prop_assert_eq!(AsRef::<str>::as_ref(AsRef::<Shown>::as_ref(&answer)), [first, second].concat());
        prop_assert_eq!(AsRef::<str>::as_ref(AsRef::<Complaint>::as_ref(&answer)), complaint);
        prop_assert_eq!(answer.code(), ExitCode::from(code));
    }

    #[test]
    fn variables_answer_from_their_pairs(name in "[A-Z]{1,8}", value in any::<String>(), other in "[a-z]{1,8}") {
        let variables: Variables = [(OsString::from(&name), OsString::from(&value))].into_iter().collect();
        prop_assert_eq!(variables.variable(OsStr::new(&name)), Some(OsString::from(value)));
        prop_assert_eq!(variables.variable(OsStr::new(&other)), None);
    }

    #[test]
    fn the_process_reads_its_variables(name in "PATH|HOME|[A-Z_]{1,12}") {
        prop_assert_eq!(Process.variable(OsStr::new(&name)), std::env::var_os(&name));
    }
}
