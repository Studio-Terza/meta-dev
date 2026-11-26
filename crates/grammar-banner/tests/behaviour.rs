#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_banner::Banner;
use proptest::prelude::{prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};

#[test]
fn a_banner_label_is_exactly_one_plain_word() {
    [
        ("grammar", Some("grammar")),
        ("doc-test_log", Some("doc-test_log")),
        ("", None),
        ("Running", None),
        ("Doc-tests", None),
        ("a b", None),
        ("a/b", None),
        ("(", None),
    ]
    .into_iter()
    .for_each(|(label, want)| {
        let read = Banner::try_new(label.to_owned()).ignoring(Unlexed::NotThisGrammar).map(Banner::into_inner);
        assert_eq!(read.as_deref(), want, "{label}");
    });
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_test_binary_banner_is_labelled_by_its_binary_and_a_doctest_banner_by_its_crate(
        name in "[a-z_][a-z0-9_-]{0,12}"
    ) {
        let running = format!("     Running unittests src/lib.rs (target/debug/deps/{name})").parse::<Banner>();
        let documented = format!("   Doc-tests {name}").parse::<Banner>();
        prop_assert_eq!(running.map(Banner::into_inner), Ok(name.clone()));
        prop_assert_eq!(documented.map(Banner::into_inner), Ok(format!("doc-{name}")));
    }
}

#[test]
fn a_banner_in_the_build_directory_layout_is_labelled_by_its_package_and_binary() {
    [
        (
            "     Running tests/behaviour.rs (target/debug/build/test-log/9b87ea0c/out/behaviour-9b87ea0c)",
            Some("test-log.behaviour"),
        ),
        (
            "     Running unittests src/lib.rs (target/release/build/graph-reader/a1b2/out/graph_reader-a1b2)",
            Some("graph-reader.graph_reader"),
        ),
        ("     Running tests/behaviour.rs (target/debug/build/test-log/9b87ea0c/out/behaviour-ffff)", None),
        ("     Running tests/behaviour.rs (target/debug/build/test-log/9b87ea0c/behaviour-9b87ea0c)", None),
    ]
    .into_iter()
    .for_each(|(line, want)| {
        let read = line.parse::<Banner>().ignoring(Unlexed::NotThisGrammar).map(Banner::into_inner);
        assert_eq!(read.as_deref(), want, "{line}");
    });
}
