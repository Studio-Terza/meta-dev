use std::io::{self, PipeReader, PipeWriter, Read, Write};
use std::process::Child;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};

use crate::{Bell, DispatchFailure, Output, Prompt, Stderr};

#[derive(Debug)]
pub(crate) struct Process {
    child: Child,
    exchange: Option<Exchange>,
}

#[derive(Debug)]
pub(crate) struct Exchange(Receiver<Result<Output, DispatchFailure>>);

impl Process {
    pub(super) fn new(child: Child, exchange: Exchange) -> Process {
        Process { child, exchange: Some(exchange) }
    }

    pub(super) fn read_out(&mut self) -> Option<Result<Output, DispatchFailure>> {
        let heard = self.exchange.as_ref().and_then(Exchange::heard);
        heard.inspect(|_heard| self.exchange = None)
    }

    pub(super) fn child(&mut self) -> &mut Child {
        &mut self.child
    }
}

impl Exchange {
    pub(crate) fn begun(
        prompt: &Prompt,
        mut prompting: PipeWriter,
        [mut replies, complaints]: [PipeReader; 2],
        bell: Bell,
    ) -> Exchange {
        let prompt = prompt.clone();
        let (telling, told) = mpsc::channel();
        let writing = thread::spawn(move || prompting.write_all(prompt.as_ref()));
        let complaining = thread::spawn(move || Stderr::heard(complaints));
        thread::spawn(move || {
            telling
                .send(Exchange::collected(&mut replies, writing, complaining))
                .into_iter()
                .for_each(|()| bell.ring());
        });
        Exchange(told)
    }

    fn collected(
        replies: &mut PipeReader,
        writing: JoinHandle<io::Result<()>>,
        complaining: JoinHandle<io::Result<Stderr>>,
    ) -> Result<Output, DispatchFailure> {
        let mut read = String::new();
        replies.read_to_string(&mut read).map_err(DispatchFailure::Collect)?;
        writing.join().map_err(|_panicked| DispatchFailure::Unjoined)?.map_err(DispatchFailure::Prompt)?;
        let stderr =
            complaining.join().map_err(|_panicked| DispatchFailure::Unjoined)?.map_err(DispatchFailure::Collect)?;
        Ok(Output::read(&read, stderr))
    }

    fn heard(&self) -> Option<Result<Output, DispatchFailure>> {
        match self.0.try_recv() {
            Ok(collected) => Some(collected),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(DispatchFailure::Unjoined)),
        }
    }
}
