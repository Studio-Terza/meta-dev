use std::fmt;

use proptest::test_runner::TestCaseError;

#[derive(Debug)]
pub(crate) struct Told(String);

impl<Failure: fmt::Display> From<&Failure> for Told {
    fn from(failure: &Failure) -> Told {
        Told(failure.to_string())
    }
}

impl From<Told> for TestCaseError {
    fn from(told: Told) -> TestCaseError {
        TestCaseError::fail(told.0)
    }
}
