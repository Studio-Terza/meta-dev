#![deny(clippy::wildcard_enum_match_arm)]

use grammar_reply::Inked;

#[test]
fn a_reply_is_text_with_ink_and_no_nul() {
    for (text, want) in [
        ("place(a.0, n1, about)", Some(21)),
        ("  x  ", Some(5)),
        ("x\ny", Some(3)),
        ("", None),
        (" \n\t", None),
        ("x\0y", None),
        ("\0", None),
    ] {
        assert_eq!(Inked::read(text).map(|inked| inked.as_ref().len()), want, "{text:?}");
    }
}
