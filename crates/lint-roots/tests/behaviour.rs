use std::path::{Path, PathBuf};

use file_ports::{Bytes, Memory};
use lint_roots::{Fact, Survey};

#[test]
fn a_survey_learns_the_canonical_form_of_each_path_it_is_asked_about() {
    let files: Memory = [(PathBuf::from("/p/src/lib.rs"), Bytes::from("struct A;\n"))].into_iter().collect();
    let asked = [PathBuf::from("/p/src/lib.rs"), PathBuf::from("/p/missing.rs")];
    let survey = Survey::learned(&files, Fact::Canonical, &asked);
    assert_eq!(
        [survey.canonical(Path::new("/p/src/lib.rs")), survey.canonical(Path::new("/p/missing.rs"))],
        [Some(Path::new("/p/src/lib.rs")), None]
    );
}
