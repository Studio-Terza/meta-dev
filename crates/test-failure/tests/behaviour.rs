use std::io;

use test_failure::Failed;

#[test]
fn a_failure_keeps_the_message_it_was_made_from() {
    let failed = Failed::from(io::Error::other("the disk is gone"));
    assert_eq!(format!("{failed:?}"), "Failed(\"the disk is gone\")");
    assert_eq!(failed, Failed::from("the disk is gone"));
}
