#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_banner::Banner;
use laws::proptest::prelude::prop_assert_eq;

laws::table!(
    a_banner_label_is_exactly_one_plain_word,
    |label: &&str| Banner::try_new((*label).to_owned()).ignoring(Unlexed::NotThisGrammar).map(Banner::into_inner),
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

laws::table!(
    a_banner_in_the_build_directory_layout_is_labelled_by_its_package_and_binary,
    |line: &&str| line.parse::<Banner>().ignoring(Unlexed::NotThisGrammar).map(Banner::into_inner),
    [
        "     Running tests/behaviour.rs (target/debug/build/test-log/9b87ea0c/out/behaviour-9b87ea0c)" => Some("test-log.behaviour".to_owned()),
        "     Running unittests src/lib.rs (target/release/build/graph-reader/a1b2/out/graph_reader-a1b2)" => Some("graph-reader.graph_reader".to_owned()),
        "     Running tests/behaviour.rs (target/debug/build/test-log/9b87ea0c/out/behaviour-ffff)" => None,
        "     Running tests/behaviour.rs (target/debug/build/test-log/9b87ea0c/behaviour-9b87ea0c)" => None,
    ]
);
