use test_log::TestLog;

const UNIT: &str = "     Running unittests src/lib.rs (target/debug/deps/foo-abc123)\n\
    test a::b ... ok\ntest c ... FAILED\ntest d ... ignored\n";
const TWO_BINARIES: &str = "Running unittests src/lib.rs (target/debug/deps/foo-1)\ntest x ... ok\n\
    Running tests/behaviour.rs (target/release/deps/behaviour-2)\ntest x ... FAILED\n";
const DOCTESTS: &str = "   Doc-tests foo\ntest src/lib.rs - bar (line 3) ... ok\n";
const BEFORE_ANY_BINARY: &str = "test x ... ok\n";

struct Sample;

impl Sample {
    fn names(text: &str) -> String {
        TestLog::from(text).names().map(|name| ["<", &name.to_string(), ">"].concat()).collect()
    }

    fn failed(text: &str) -> String {
        TestLog::from(text).failed().into_iter().map(|name| ["<", &name.to_string(), ">"].concat()).collect()
    }
}

#[test]
fn a_binary_is_named_by_its_running_line_and_labels_the_tests_after_it() {
    for (text, want) in [
        (UNIT, "<foo-abc123 :: a::b><foo-abc123 :: c>"),
        (TWO_BINARIES, "<behaviour-2 :: x><foo-1 :: x>"),
        (DOCTESTS, "<doc-foo :: src/lib.rs - bar (line 3)>"),
        (BEFORE_ANY_BINARY, "< :: x>"),
        ("Running x (target/debug/deps/n)\ntest t ... ok\n", "<n :: t>"),
        ("Running x (target/ignored) (target/debug/deps/last)\ntest t ... ok\n", "<last :: t>"),
        ("Doc-tests a b\ntest t ... ok\n", "<doc-a :: t>"),
        (
            "Running tests/behaviour.rs (target/debug/build/graph/0a1b/out/behaviour-0a1b)\ntest t ... ok\n",
            "<graph.behaviour :: t>",
        ),
    ] {
        assert_eq!(Sample::names(text), want, "{text:?}");
    }
}

#[test]
fn a_line_that_is_not_a_running_line_keeps_the_previous_binary() {
    for second in [
        "note: Running b (target/debug/deps/two)",
        "Running b (target/a/b/deps/two)",
        "Running b (target//deps/two)",
        "Running b (target/debug/deps/)",
        "Running b (target/debug/deps/t w)",
        "Running b (target/debug/deps/two",
        "Running b",
        "Runningb (target/debug/deps/two)",
        "Doc-tests",
        "Doc-tests   ",
        "note: Doc-tests z",
        "Doc-testsz",
    ] {
        let text = format!("Running a (target/debug/deps/one)\n{second}\ntest t ... ok\n");
        assert_eq!(Sample::names(&text), "<one :: t>", "{second:?}");
    }
}

#[test]
fn only_whole_test_lines_with_a_verdict_are_results() {
    for (text, want) in [
        ("test a ... ok\n", "< :: a>"),
        ("test a ... ok  \n", "< :: a>"),
        ("test a ... ok\t\n", "< :: a>"),
        ("test a ... FAILED\n", "< :: a>"),
        ("test a ... ignored\n", ""),
        ("test a ... bench\n", ""),
        ("test a ... okay\n", ""),
        ("test a ...ok\n", ""),
        ("test a...ok\n", ""),
        ("  test a ... ok\n", ""),
        ("tests a ... ok\n", ""),
        ("testa ... ok\n", ""),
        ("test  ... ok\n", ""),
        ("test ... ok\n", ""),
        ("test\n", ""),
        ("test a ... b ... ok\n", "< :: a ... b>"),
        ("test a ... ok ... ok\n", "< :: a ... ok>"),
        ("test a ... ok\ntest a ... ok\n", "< :: a>"),
        ("test a ... ok", "< :: a>"),
        ("test ü ... ok\n", "< :: ü>"),
        ("", ""),
        ("\n\n", ""),
    ] {
        assert_eq!(Sample::names(text), want, "{text:?}");
    }
}

#[test]
fn a_failed_verdict_counts_only_when_it_is_the_last_for_the_name() {
    for (text, want) in [
        (UNIT, "<foo-abc123 :: c>"),
        (TWO_BINARIES, "<behaviour-2 :: x>"),
        ("test a ... FAILED\ntest a ... ok\n", ""),
        ("test a ... ok\ntest a ... FAILED\n", "< :: a>"),
        ("test a ... ok\n", ""),
        ("", ""),
    ] {
        assert_eq!(Sample::failed(text), want, "{text:?}");
    }
}
