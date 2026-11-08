mod extension;

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::PathBuf;

use file_ports::{FileFailure, Files};
use newtype::Newtype;
use port_core::Pass;

use crate::transcripts::{TranscriptsFiller, transcripts_ops};
use crate::{Cursor, Read, ReadFailure, Session, SessionId};
use extension::Extension;

#[derive(Debug)]
pub struct ClaudeCodeDir<F: Files> {
    files: F,
    directory: PathBuf,
}

impl<F: Files> ClaudeCodeDir<F> {
    pub fn new(files: F, directory: PathBuf) -> ClaudeCodeDir<F> {
        ClaudeCodeDir { files, directory }
    }

    fn session_at(&self, path: PathBuf) -> Option<Result<Session, FileFailure>> {
        path.extension()
            .and_then(OsStr::to_str)
            .and_then(|named| named.parse::<Extension>().ok())
            .and(path.file_stem())
            .map(|stem| SessionId::new(stem.to_string_lossy().into_owned()))
            .map(|id| self.files.size(&path).map(|size| Session::found(id, path, size)))
    }
}

impl<F: Files> TranscriptsFiller for ClaudeCodeDir<F> {
    fn sessions(&self, _pass: Pass<'_, transcripts_ops::Sessions>) -> Result<Vec<Session>, FileFailure> {
        let sessions = self
            .files
            .entries(&self.directory)?
            .into_iter()
            .filter_map(|path| self.session_at(path))
            .collect::<Result<BTreeSet<Session>, FileFailure>>()?;
        Ok(sessions.into_iter().collect())
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
