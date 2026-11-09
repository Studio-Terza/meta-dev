#![deny(clippy::wildcard_enum_match_arm)]

use grammar_banner::Banner;
use laws::proptest::prelude::prop_assert_eq;

laws::table!(
    a_banner_label_is_exactly_one_plain_word,
    |label: &&str| Banner::try_new((*label).to_owned()).ok().map(Banner::into_inner),
    [
        "grammar" => Some("grammar".to_owned()),
        "doc-test_log" => Some("doc-test_log".to_owned()),
        "" => None,
        "Running" => None,
        "Doc-tests" => None,
        "a b" => None,
        "a/b" => None,
        "(" => None,
    ]
);

laws::law!(
    a_test_binary_banner_is_labelled_by_its_binary_and_a_doctest_banner_by_its_crate,
    "[a-z_][a-z0-9_-]{0,12}",
    name,
    {
        let running = format!("     Running unittests src/lib.rs (target/debug/deps/{name})").parse::<Banner>();
        let documented = format!("   Doc-tests {name}").parse::<Banner>();
        prop_assert_eq!(running.map(Banner::into_inner), Ok(name.clone()));
        prop_assert_eq!(documented.map(Banner::into_inner), Ok(format!("doc-{name}")));
    }
);
