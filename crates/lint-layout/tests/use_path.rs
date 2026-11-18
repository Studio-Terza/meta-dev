use lint_layout::{Name, UsePath};

#[test]
fn a_path_names_up_to_and_including_its_first_typed_segment() {
    let path: UsePath = ["a", "b", "c", "d"].into_iter().map(Name::from).collect();
    let named = path.naming(|segment| Some(segment).filter(|found| found.to_string() == "b"));
    assert_eq!(named.to_string(), "a::b");
}
