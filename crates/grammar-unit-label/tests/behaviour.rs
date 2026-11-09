#![deny(clippy::wildcard_enum_match_arm)]

use grammar_unit_label::{Fixed, Target, Unit};
use laws::proptest::sample::select;

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
    fn read(text: &&str) -> Option<Described> {
        Described::labelled(text)
    }

    fn targeted(text: &str) -> Described {
        Described::Target(text.to_owned())
    }
}

laws::table!(
    a_unit_label_reads_as_the_role_cargo_gave_the_unit,
    Described::read,
    [
        "" => Some(Described::Fixed(Fixed::Built)),
        " (check)" => Some(Described::Fixed(Fixed::Checked)),
        " (check-test)" => Some(Described::Fixed(Fixed::CheckedTest)),
        " build-script" => Some(Described::Fixed(Fixed::BuildScript)),
        " build-script (run)" => Some(Described::Fixed(Fixed::BuildRun)),
        " app \"bin\" (check)" => Some(Described::targeted(" app \"bin\" (check) Check")),
        " behaviour \"test\" (check-test)" => Some(Described::targeted(" behaviour \"test\" (check-test) CheckTest")),
        " b \"bench\" (check-test)" => Some(Described::targeted(" b \"bench\" (check-test) CheckTest")),
        " ex \"example\" (check)" => Some(Described::targeted(" ex \"example\" (check) Check")),
        " my-tool_2 \"bin\" (check-test)" => Some(Described::targeted(" my-tool_2 \"bin\" (check-test) CheckTest")),
    ]
);

laws::table!(
    a_label_cargo_does_not_write_for_a_clippy_unit_is_refused,
    Described::read,
    [
        "(check)" => None,
        " (check) " => None,
        "  (check)" => None,
        " (run)" => None,
        " build-script (check)" => None,
        " (check) build-script" => None,
        " app bin (check)" => None,
        " app \"bin\"" => None,
        " app \"lib\" (check)" => None,
        " app \"bin\" (run)" => None,
        " app \"bin\" (doc)" => None,
        "app \"bin\" (check)" => None,
    ]
);

laws::roundtrip!(
    a_fixed_label_is_displayed_and_parsed_back,
    select(vec![Fixed::Built, Fixed::Checked, Fixed::CheckedTest, Fixed::BuildScript, Fixed::BuildRun]),
    |fixed: &Fixed| fixed.to_string(),
    |text: &String| text.parse::<Fixed>().ok(),
    Some
);
