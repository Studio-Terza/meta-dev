#![deny(clippy::wildcard_enum_match_arm)]

use grammar_ident::Words;

#[test]
fn camel_names_split_into_lowered_humps() {
    assert_eq!(Words::lowered_humps("Transcripts"), vec!["transcripts"]);
    assert_eq!(Words::lowered_humps("ReadFrom"), vec!["read", "from"]);
    assert_eq!(Words::lowered_humps("lowerThenUpper"), vec!["lower", "then", "upper"]);
    assert_eq!(Words::lowered_humps(""), Vec::<String>::new());
}

#[test]
fn snake_names_raise_into_camel_case() {
    assert_eq!(Words::camel("read_from"), "ReadFrom");
    assert_eq!(Words::camel("sessions"), "Sessions");
    assert_eq!(Words::camel("a__b_"), "AB");
    assert_eq!(Words::camel(""), "");
}
