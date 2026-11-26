use std::path::{Path, PathBuf};

use file_ports::{Bytes, Memory};
use lint_roots::{Fact, Roots, Survey, Tree};

#[test]
fn a_listing_failure_is_reported_once_to_the_first_walk_that_reaches_it() {
    let files: Memory = [(PathBuf::from("/p/lib.rs"), Bytes::from("struct A;\n"))].into_iter().collect();
    let given = [PathBuf::from("/p/lib.rs"), PathBuf::from("/p/lib.rs")];
    let opened = Tree::open_many(&files, &given);
    let trees: Vec<&Tree<'_>> = opened.iter().flatten().collect();
    let reported = Roots::surveyed(&files, &trees).trees().iter().filter(|walked| walked.is_err()).count();
    assert_eq!(reported, 2);
}

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
