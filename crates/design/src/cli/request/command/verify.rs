mod boundary;

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use console_ports::Shown;
use design_messages::Tally;
use dispatch_ports::{Clock, Dispatch, Model, Prompt, Timeout};
use file_ports::{Bytes, Files};
use store_ports::Store;
use transcript_ports::{Body, Uuid};

use super::Ground;
use super::backtest::{Log, Spoken};
use super::crew::{Crew, Heard};
use super::pairs::{Pair, PairLines};
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::{Answered, Check};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verify {
    session: PathBuf,
    pairs: PathBuf,
    verified: PathBuf,
}

#[derive(Debug, Default)]
struct Verified {
    checked: usize,
    kept: usize,
    rejected: usize,
    unread: usize,
    tokens: Tally,
}

impl Verify {
    const TIMEOUT: Timeout = Timeout::new(300);
    const FAN: usize = 16;

    pub fn new([session, pairs, verified]: [&&str; 3]) -> Verify {
        Verify { session: PathBuf::from(session), pairs: PathBuf::from(pairs), verified: PathBuf::from(verified) }
    }

    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        let files = ground.files();
        let log = Log::read(files.read_text(&self.session)?.as_ref());
        let pairs = Pair::read(files.read_text(&self.pairs)?.as_ref());
        let owned: BTreeMap<&Uuid, &Body> = log
            .spoken()
            .iter()
            .filter_map(|spoken| match spoken {
                Spoken::Owner { uuid, said } => Some((uuid, said)),
                Spoken::Lead(_) => None,
            })
            .collect();
        let checks: Vec<Check<'_>> =
            pairs.iter().filter_map(|pair| owned.get(pair.answer()).map(|answer| Check::new(pair, answer))).collect();
        files.write(&self.verified, Bytes::default())?;
        let batches = checks
            .chunks(Verify::FAN)
            .map(|batch| self.judged(files, crew, batch))
            .collect::<Result<Vec<Verified>, Failure>>()?;
        let verified = Verified {
            checked: batches.iter().map(|batch| batch.checked).sum(),
            kept: batches.iter().map(|batch| batch.kept).sum(),
            rejected: batches.iter().map(|batch| batch.rejected).sum(),
            unread: batches.iter().map(|batch| batch.unread).sum(),
            tokens: batches.iter().map(|batch| batch.tokens).sum(),
        };
        Ok(Uttered::new(Shown::said(verified), Ok(Outcome::Verified)))
    }

    fn judged<D: Dispatch, C: Clock>(
        &self,
        files: &impl Files,
        crew: &Crew<'_, D, C>,
        batch: &[Check<'_>],
    ) -> Result<Verified, Failure> {
        let prompts: Vec<Prompt> = batch.iter().map(Prompt::from).collect();
        let heard = crew.fanned_to(&prompts, Verify::TIMEOUT, Model::Sonnet)?;
        let verdicts: Vec<Option<Answered>> = heard.iter().map(Check::judged).collect();
        let kept: Vec<Pair> = batch
            .iter()
            .zip(&verdicts)
            .filter(|(_check, verdict)| **verdict == Some(Answered::Yes))
            .map(|(check, _verdict)| check.pair().clone())
            .collect();
        files.append(&self.verified, &Bytes::from(&PairLines::from(kept.as_slice())))?;
        Ok(Verified {
            checked: batch.len(),
            kept: kept.len(),
            rejected: verdicts.iter().filter(|verdict| **verdict == Some(Answered::No)).count(),
            unread: verdicts.iter().filter(|verdict| verdict.is_none()).count(),
            tokens: heard.iter().map(Heard::tokens).sum(),
        })
    }
}

impl fmt::Display for Verified {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "checked\tkept\trejected\tunread\ttokens\n{}\t{}\t{}\t{}\t{}",
            self.checked, self.kept, self.rejected, self.unread, self.tokens
        )
    }
}
