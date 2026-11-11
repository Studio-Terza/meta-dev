use std::io::{self, PipeReader, Read};

use grammar_reply::Inked;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Stderr {
    #[error("it wrote nothing on stderr")]
    Silent,
    #[error("on stderr it wrote {0:?}")]
    Wrote(String),
}

impl Stderr {
    const KEPT: u64 = 4096;

    pub(crate) fn heard(mut complaints: PipeReader) -> io::Result<Stderr> {
        let mut kept = Vec::new();
        (&mut complaints).take(Self::KEPT).read_to_end(&mut kept)?;
        io::copy(&mut complaints, &mut io::sink())?;
        let text: String = kept.utf8_chunks().map(|chunk| chunk.valid()).collect();
        Ok(Inked::read(&text).map_or(Stderr::Silent, |said| Stderr::Wrote(said.as_ref().to_owned())))
    }
}
