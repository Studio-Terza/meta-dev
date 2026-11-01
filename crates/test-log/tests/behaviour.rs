use test_log::TestLog;

const UNIT: &str = "     Running unittests src/lib.rs (target/debug/deps/foo-abc123)\n\
    test a::b ... ok\ntest c ... FAILED\ntest d ... ignored\n";
const TWO_BINARIES: &str = "Running unittests src/lib.rs (target/debug/deps/foo-1)\ntest x ... ok\n\
    Running tests/behaviour.rs (target/release/deps/behaviour-2)\ntest x ... FAILED\n";
const DOCTESTS: &str = "   Doc-tests foo\ntest src/lib.rs - bar (line 3) ... ok\n";
const BEFORE_ANY_BINARY: &str = "test x ... ok\n";

struct Sample;

impl Sample {
    fn names(text: &&str) -> String {
        TestLog::from(*text).names().into_iter().map(|name| ["<", &name, ">"].concat()).collect()
    }

    fn failed(text: &&str) -> String {
        TestLog::from(*text).failed().into_iter().map(|name| ["<", &name, ">"].concat()).collect()
    }
}

laws::table!(
    a_binary_is_named_by_its_running_line_and_labels_the_tests_after_it,
    Sample::names,
    [
        UNIT => "<foo-abc123 :: a::b><foo-abc123 :: c>",
        TWO_BINARIES => "<behaviour-2 :: x><foo-1 :: x>",
        DOCTESTS => "<doc-foo :: src/lib.rs - bar (line 3)>",
        BEFORE_ANY_BINARY => "< :: x>",
        "Running x (target/debug/deps/n)\ntest t ... ok\n" => "<n :: t>",
        "Running x (target/ignored) (target/debug/deps/last)\ntest t ... ok\n" => "<last :: t>",
        "Doc-tests a b\ntest t ... ok\n" => "<doc-a :: t>",
    ]
);

laws::table!(
    a_line_that_is_not_a_running_line_keeps_the_previous_binary,
    Sample::names,
    [
        "Running a (target/debug/deps/one)\nnote: Running b (target/debug/deps/two)\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunning b (target/a/b/deps/two)\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunning b (target//deps/two)\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunning b (target/debug/deps/)\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunning b (target/debug/deps/t w)\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunning b (target/debug/deps/two\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunning b\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nRunningb (target/debug/deps/two)\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nDoc-tests\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nnote: Doc-tests z\ntest t ... ok\n" => "<one :: t>",
        "Running a (target/debug/deps/one)\nDoc-testsz\ntest t ... ok\n" => "<one :: t>",
    ]
);

laws::table!(
    only_whole_test_lines_with_a_verdict_are_results,
    Sample::names,
    [
        "test a ... ok\n" => "< :: a>",
        "test a ... ok  \n" => "< :: a>",
        "test a ... ok\t\n" => "< :: a>",
        "test a ... FAILED\n" => "< :: a>",
        "test a ... ignored\n" => "",
        "test a ... bench\n" => "",
        "test a ... okay\n" => "",
        "test a ...ok\n" => "",
        "test a...ok\n" => "",
        "  test a ... ok\n" => "",
        "tests a ... ok\n" => "",
        "testa ... ok\n" => "",
        "test  ... ok\n" => "",
        "test ... ok\n" => "",
        "test\n" => "",
        "test a ... b ... ok\n" => "< :: a ... b>",
        "test a ... ok ... ok\n" => "< :: a ... ok>",
        "test a ... ok\ntest a ... ok\n" => "< :: a>",
        "test a ... ok" => "< :: a>",
        "test ü ... ok\n" => "< :: ü>",
        "" => "",
        "\n\n" => "",
    ]
);

laws::table!(
    a_failed_verdict_counts_only_when_it_is_the_last_for_the_name,
    Sample::failed,
    [
        UNIT => "<foo-abc123 :: c>",
        TWO_BINARIES => "<behaviour-2 :: x>",
        "test a ... FAILED\ntest a ... ok\n" => "",
        "test a ... ok\ntest a ... FAILED\n" => "< :: a>",
        "test a ... ok\n" => "",
        "" => "",
    ]
);
