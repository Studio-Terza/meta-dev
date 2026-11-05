use std::cmp::Ordering;
use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Exit {
    Succeeded,
    Failed,
}

impl Exit {
    const SUCCESS: i32 = 0;

    fn coded(code: i32) -> Exit {
        match code.cmp(&Self::SUCCESS) {
            Ordering::Equal => Exit::Succeeded,
            Ordering::Less | Ordering::Greater => Exit::Failed,
        }
    }
}

impl From<Option<i32>> for Exit {
    fn from(code: Option<i32>) -> Exit {
        match code {
            Some(code) => Exit::coded(code),
            None => Exit::Failed,
        }
    }
}

impl From<ExitStatus> for Exit {
    fn from(status: ExitStatus) -> Exit {
        Exit::from(status.code())
    }
}
