use std::path::Path;

use lint_layout::{InFile, LineNumber, Site};

#[test]
fn a_site_names_its_file_and_line_and_moves_to_another_file() {
    let site = Site::new(Path::new("src/a.rs"), LineNumber::from(7));
    assert_eq!(site.to_string(), "src/a.rs:7");
    assert_eq!(site.path(), Path::new("src/a.rs"));
    assert_eq!(site.moved(Path::new("src/b.rs")).to_string(), "src/b.rs:7");
    assert_eq!(Site::of_file(Path::new("src/c.rs")).to_string(), "src/c.rs:1");
}

#[test]
fn any_value_in_a_file_keeps_its_value_and_path() {
    let held = InFile::new(Path::new("src/a.rs"), 'x');
    assert_eq!(*AsRef::<char>::as_ref(&held), 'x');
    assert_eq!(held.moved(Path::new("src/b.rs")), InFile::new(Path::new("src/b.rs"), 'x'));
    assert!(InFile::new(Path::new("src/a.rs"), 'b') < InFile::new(Path::new("src/b.rs"), 'a'));
}
