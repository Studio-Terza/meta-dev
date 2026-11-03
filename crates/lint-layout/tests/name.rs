use laws::proptest::prop_assert;
use lint_layout::{Casing, Lowered, Name, Word};

laws::law!(a_name_equals_the_text_it_was_made_from, "[a-zA-Z_]{1,12}", text, {
    let name = Name::from(text.as_str());
    prop_assert!(name == text.as_str());
});

laws::law!(a_lowered_name_equals_its_text_in_lower_case, "[a-zA-Z_]{1,12}", text, {
    let lower: String = text.chars().flat_map(char::to_lowercase).collect();
    let lowered: Lowered = Name::from(text.as_str()).lowered();
    prop_assert!(lowered == Word::new(&lower));
});

laws::table!(a_name_is_cased_by_its_letters, |text: &&str| Name::from(*text).casing(), [
    "snake_case" => Casing::Lower,
    "CamelCase" => Casing::Camel,
    "A1" => Casing::Camel,
    "SHOUTING" => Casing::Shouting,
    "_hidden" => Casing::Other,
]);

laws::table!(a_name_splits_into_lowered_humps, |text: &&str| Name::from(*text).humps().map(|hump| hump.to_string()).collect::<Vec<_>>().join(" "), [
    "PortTrait" => "port trait".to_owned(),
    "read_all_files" => "read all files".to_owned(),
    "HTTPServer" => "http server".to_owned(),
]);
