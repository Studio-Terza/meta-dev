use laws::proptest::prop_assert;
use lint_layout::{Casing, Name};

laws::law!(a_name_equals_the_text_it_was_made_from, "[a-zA-Z_]{1,12}", text, {
    let name = Name::from(text.as_str());
    prop_assert!(name == text.as_str());
});

laws::table!(a_name_is_cased_by_its_letters, |text: &&str| Name::from(*text).casing(), [
    "snake_case" => Casing::Lower,
    "CamelCase" => Casing::Camel,
    "A1" => Casing::Camel,
    "SHOUTING" => Casing::Shouting,
    "_hidden" => Casing::Other,
]);
