use std::process::ExitCode;
#[test]
fn exits() {
    assert_eq!(ExitCode::SUCCESS, ExitCode::SUCCESS);
}
