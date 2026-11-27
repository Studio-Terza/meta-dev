use std::io::{self, Read};
use std::process::ChildStdout;
use std::thread::{self, JoinHandle};

use super::SystemShell;
use crate::{DispatchFailure, Printed};

impl SystemShell {
    pub(super) fn read(mut stdout: ChildStdout) -> JoinHandle<io::Result<String>> {
        thread::spawn(move || {
            let mut text = String::new();
            stdout.read_to_string(&mut text).map(|_read| text)
        })
    }

    pub(super) fn collected(reading: Option<JoinHandle<io::Result<String>>>) -> Result<Printed, DispatchFailure> {
        let text = reading
            .map(|handle| handle.join().map_err(|_panicked| DispatchFailure::Unjoined))
            .transpose()?
            .transpose()
            .map_err(DispatchFailure::Collect)?;
        Ok(Printed::new(text.unwrap_or_default()))
    }
}
