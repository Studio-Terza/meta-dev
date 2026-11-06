use std::process::{ExitCode, Termination};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Shown,
}

impl Termination for Outcome {
    fn report(self) -> ExitCode {
        match self {
            Outcome::Ingested | Outcome::Shown => ().report(),
        }
    }
}
