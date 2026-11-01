use laws::proptest::prop_assert;
use lint_layout::Name;

laws::law!(a_name_equals_the_text_it_was_made_from, "[a-zA-Z_]{1,12}", text, {
    let name = Name::from(text.as_str());
    prop_assert!(name == text.as_str());
});
