const REPORT: &str = "a\nb\n";

#[test]
fn pins_a_report() {
    assert_eq!(corpus::render(), REPORT);
}

#[test]
fn pins_debug_text() {
    assert_ne!(format!("{:?}", corpus::rows()), corpus::render());
}

#[test]
fn asserts_on_the_parts() {
    assert_eq!(corpus::lines(), ["a", "b"]);
}

laws::table!(rendered_rows, corpus::row, [1 => "a\nb"]);
