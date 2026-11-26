#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_unit_label::{Fixed, Target, Unit};
use strum::VariantArray;

#[derive(Debug, PartialEq, Eq)]
enum Described {
    Fixed(Fixed),
    Target(String),
}

impl Unit for Described {
    fn fixed(fixed: Fixed) -> Described {
        Described::Fixed(fixed)
    }

    fn target(target: Target) -> Described {
        Described::Target(format!("{target} {:?}", target.mode()))
    }
}

impl Described {
    fn targeted(text: &str) -> Described {
        Described::Target(text.to_owned())
    }
}

#[test]
fn a_unit_label_reads_as_the_role_cargo_gave_the_unit() {
    for (label, want) in [
        ("", Described::Fixed(Fixed::Built)),
        (" (check)", Described::Fixed(Fixed::Checked)),
        (" (check-test)", Described::Fixed(Fixed::CheckedTest)),
        (" build-script", Described::Fixed(Fixed::BuildScript)),
        (" build-script (run)", Described::Fixed(Fixed::BuildRun)),
        (" app \"bin\" (check)", Described::targeted(" app \"bin\" (check) Check")),
        (" behaviour \"test\" (check-test)", Described::targeted(" behaviour \"test\" (check-test) CheckTest")),
        (" b \"bench\" (check-test)", Described::targeted(" b \"bench\" (check-test) CheckTest")),
        (" ex \"example\" (check)", Described::targeted(" ex \"example\" (check) Check")),
        (" my-tool_2 \"bin\" (check-test)", Described::targeted(" my-tool_2 \"bin\" (check-test) CheckTest")),
    ] {
        assert_eq!(Described::labelled(label), Some(want), "{label}");
    }
}

#[test]
fn a_label_cargo_does_not_write_for_a_clippy_unit_is_refused() {
    for label in [
        "(check)",
        " (check) ",
        "  (check)",
        " (run)",
        " build-script (check)",
        " (check) build-script",
        " app bin (check)",
        " app \"bin\"",
        " app \"lib\" (check)",
        " app \"bin\" (run)",
        " app \"bin\" (doc)",
        "app \"bin\" (check)",
    ] {
        assert_eq!(Described::labelled(label), None, "{label}");
    }
}

#[test]
fn a_fixed_label_is_displayed_and_parsed_back() {
    for fixed in Fixed::VARIANTS {
        assert_eq!(fixed.to_string().parse::<Fixed>().ignoring(Unlexed::NotThisGrammar), Some(*fixed));
    }
}
