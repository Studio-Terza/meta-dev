#![deny(clippy::wildcard_enum_match_arm)]

use grammar_program::Program;
use laws::proptest::prelude::any;

laws::roundtrip!(
    a_command_s_program_is_its_first_word_whatever_follows,
    (any::<Program>(), "[ \t]{0,3}", "([ \t][ -~]{0,20})?"),
    |(program, lead, rest): &(Program, String, String)| format!("{lead}{}{rest}", program.clone().into_inner()),
    |text: &String| text.parse::<Program>().ok(),
    |(program, _lead, _rest): (Program, String, String)| Some(program)
);

laws::table!(
    a_program_holds_exactly_one_word_with_no_space_around_it,
    |word: &&str| Program::try_new((*word).to_owned()).ok().map(Program::into_inner),
    [
        "cargo" => Some("cargo".to_owned()),
        "./run.sh" => Some("./run.sh".to_owned()),
        "" => None,
        "cargo test" => None,
        " cargo" => None,
        "cargo\n" => None,
    ]
);
