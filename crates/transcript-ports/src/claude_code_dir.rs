use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files};
use newtype::Newtype;
use port_core::Pass;

use crate::transcripts::{TranscriptsFiller, transcripts_ops};
use crate::{Cursor, Read, ReadFailure, Session, SessionId};

#[derive(Debug)]
pub struct ClaudeCodeDir<F: Files> {
    files: F,
    directory: PathBuf,
}

impl<F: Files> ClaudeCodeDir<F> {
    pub fn new(files: F, directory: PathBuf) -> ClaudeCodeDir<F> {
        ClaudeCodeDir { files, directory }
    }

    fn session_at(path: PathBuf) -> Option<Session> {
        Some(path.as_path())
            .filter(|file| file.extension() == Some(OsStr::new("jsonl")))
            .and_then(Path::file_stem)
            .map(|stem| SessionId::new(stem.to_string_lossy().into_owned()))
            .map(|id| Session::found(id, path))
    }
}

impl<F: Files> TranscriptsFiller for ClaudeCodeDir<F> {
    fn sessions(&self, _pass: Pass<'_, transcripts_ops::Sessions>) -> Result<Vec<Session>, FileFailure> {
        let mut sessions: Vec<Session> =
            self.files.entries(&self.directory)?.into_iter().filter_map(Self::session_at).collect();
        sessions.sort();
        Ok(sessions)
    }

    fn read_from(
        &self,
        _pass: Pass<'_, transcripts_ops::ReadFrom>,
        file: &Session,
        cursor: Cursor,
    ) -> Result<Read, ReadFailure> {
        let bytes = self.files.read_from(file.file(), cursor.into_inner())?;
        Read::complete(bytes.as_ref(), cursor)
    }
}
