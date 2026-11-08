use std::io::{PipeReader, PipeWriter, Read, Write};
use std::process::Child;
use std::thread::{self, JoinHandle};

use crate::{DispatchFailure, Prompt};

#[derive(Debug)]
pub(crate) struct Process {
    child: Child,
    exchange: Option<Exchange>,
}

#[derive(Debug)]
pub(crate) struct Exchange(JoinHandle<Result<String, DispatchFailure>>);

impl Process {
    pub(super) fn new(child: Child, exchange: Exchange) -> Process {
        Process { child, exchange: Some(exchange) }
    }

    pub(super) fn read_out(&mut self) -> Option<Exchange> {
        self.exchange.take_if(|exchange| exchange.0.is_finished())
    }

    pub(super) fn child(&mut self) -> &mut Child {
        &mut self.child
    }
}

impl Exchange {
    pub(crate) fn begun(prompt: &Prompt, mut prompting: PipeWriter, mut replies: PipeReader) -> Exchange {
        let prompt = prompt.clone();
        let writing = thread::spawn(move || prompting.write_all(prompt.as_ref()));
        Exchange(thread::spawn(move || {
            let mut read = String::new();
            replies.read_to_string(&mut read).map_err(DispatchFailure::Collect)?;
            writing.join().map_err(|_panicked| DispatchFailure::Unjoined)?.map_err(DispatchFailure::Prompt)?;
            Ok(read)
        }))
    }

    pub(crate) fn joined(self) -> Result<String, DispatchFailure> {
        self.0.join().map_err(|_panicked| DispatchFailure::Unjoined)?
    }
}
