use grammar_lines::{Fields, KeptLines, Lines};

#[test]
fn text_splits_into_lines_with_and_without_their_breaks_and_into_tab_fields() {
    let split = |text: &str| {
        let lines: Vec<String> = text.parse::<Lines>().into_iter().flatten().collect();
        let kept: Vec<String> = text.parse::<KeptLines>().into_iter().flatten().collect();
        let fields: Vec<Vec<String>> = lines
            .iter()
            .flat_map(|line| line.parse::<Fields>().into_iter().map(|cells| cells.as_ref().to_vec()))
            .collect();
        format!("{lines:?} {kept:?} {fields:?}")
    };
    assert_eq!(split(""), "[] [] []");
    assert_eq!(
        split("a\tb\n\nc\r\nd"),
        r#"["a\tb", "", "c\r", "d"] ["a\tb\n", "\n", "c\r\n", "d"] [["a", "b"], [], ["c\r"], ["d"]]"#
    );
    assert_eq!(split("\t\tx\t\n"), r#"["\t\tx\t"] ["\t\tx\t\n"] [["x"]]"#);
}
