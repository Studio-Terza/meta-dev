use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files};

use super::verdict::Verdict;
use crate::cli::command::Telling;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports};

#[derive(Debug)]
pub(super) struct Keeping(Option<PathBuf>);

impl From<Option<PathBuf>> for Keeping {
    fn from(directory: Option<PathBuf>) -> Keeping {
        Keeping(directory)
    }
}

impl Keeping {
    const REVISIONS: [&'static str; 3] = ["rev-parse", "HEAD", "HEAD^2"];
    const TESTED: &'static str = "tested";
    const OUTCOMES: &'static str = "outcomes.json";

    pub(super) fn concluded<'step>(
        &self,
        ports: &impl Ports,
        verdict: Verdict<'_>,
    ) -> Result<Telling<'step>, Failure<'step>> {
        self.kept(ports, None).map(|()| Telling::listed(verdict.told()))
    }

    pub(super) fn kept(&self, ports: &impl Ports, outcomes: Option<&Path>) -> Result<(), Failure<'static>> {
        self.0.as_ref().map_or(Ok(()), |directory| Self::written(ports, directory, outcomes))
    }

    fn written(ports: &impl Ports, directory: &Path, outcomes: Option<&Path>) -> Result<(), Failure<'static>> {
        let invocation = Invocation::new("git", Self::REVISIONS);
        let tested = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let files = ports.files();
        let copied = outcomes
            .map(|path| files.read_text(path).map_err(|failure| Failure::unusable(path.to_path_buf(), failure)))
            .transpose()?;
        files.create_directories(directory).map_err(|failure| Failure::unusable(directory.to_path_buf(), failure))?;
        let kept = directory.join(Self::OUTCOMES);
        copied
            .map(|text| files.write(&kept, Bytes::from(text.as_ref())))
            .transpose()
            .map_err(|failure| Failure::unusable(kept.clone(), failure))?;
        let path = directory.join(Self::TESTED);
        files.write(&path, Bytes::from(tested.as_ref())).map_err(|failure| Failure::unusable(path, failure))
    }
}
