#![deny(clippy::wildcard_enum_match_arm)]

use grammar_slug::Letters;

#[test]
fn slugs_keep_letters_and_digits_and_replace_every_other_character() {
    assert_eq!(Letters::slugged("/Users/me/my_repo"), "-Users-me-my-repo");
    assert_eq!(Letters::slugged("abc123"), "abc123");
    assert_eq!(Letters::slugged("a.b é"), "a-b--");
    assert_eq!(Letters::slugged(""), "");
}
