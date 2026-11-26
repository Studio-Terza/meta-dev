use lint_layout::{Casing, Lowered, Name};
use proptest::prelude::{prop_assert, proptest};
use proptest::test_runner::{Config, RngSeed};

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_name_displays_the_text_it_was_made_from(text in "[a-zA-Z_]{1,12}") {
        let name = Name::from(text.as_str());
        prop_assert!(name.to_string() == text);
    }

    #[test]
    fn lowering_a_name_ignores_its_case(text in "[a-zA-Z_]{1,12}") {
        let lower: String = text.chars().flat_map(char::to_lowercase).collect();
        let lowered: Lowered = Name::from(text.as_str()).lowered();
        prop_assert!(lowered == Name::from(lower.as_str()).lowered());
    }
}

#[test]
fn a_name_is_cased_by_its_letters() {
    for (text, want) in [
        ("snake_case", Casing::Lower),
        ("CamelCase", Casing::Camel),
        ("A1", Casing::Camel),
        ("SHOUTING", Casing::Shouting),
        ("_hidden", Casing::Other),
    ] {
        assert_eq!(Name::from(text).casing(), want, "{text}");
    }
}
