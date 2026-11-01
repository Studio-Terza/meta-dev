use lint_core::Lint;
use lint_fixtures::Sources;
use lint_no_comments::NoComments;

const COMMENTED: &str = "fn a() {}\n// line\nfn b() {} /* block */\n";
const PLAIN: &str = "const TEXT: &str = \"// not a comment\";\n";

#[test]
fn comments_are_reported_with_their_lines_here() {
    let report = NoComments::report(&[Sources::file("a.rs", COMMENTED)]);
    assert_eq!(Sources::lines(&report), ["a.rs:2", "a.rs:3"]);
}

#[test]
fn comment_markers_inside_strings_are_not_comments() {
    assert!(NoComments::report(&[Sources::file("a.rs", PLAIN)]).is_clean());
}
