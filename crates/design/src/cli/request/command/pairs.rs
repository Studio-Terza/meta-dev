mod boundary;

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

use console_ports::Shown;
use design_messages::Tally;
use dispatch_ports::{Clock, Dispatch, Prompt, Timeout};
use file_ports::{Bytes, Files};
use store_ports::Store;

use super::Ground;
use super::backtest::Log;
use super::crew::{Crew, Heard};
use super::walk::Walk;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::{Excerpt, Turn};
pub use boundary::{Pair, PairLines};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pairs {
    tree: PathBuf,
    session: PathBuf,
    written: PathBuf,
}

#[derive(Debug)]
struct Extracted {
    excerpts: usize,
    pairs: usize,
    tokens: Tally,
}

impl Pairs {
    const TIMEOUT: Timeout = Timeout::new(300);
    const FAN: usize = 16;

    pub fn new([tree, session, written]: [&&str; 3]) -> Pairs {
        Pairs { tree: PathBuf::from(tree), session: PathBuf::from(session), written: PathBuf::from(written) }
    }

    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        let files = ground.files();
        let tree = Walk::planted(files, &self.tree)?;
        let known = tree.messages();
        let log = Log::read(files.read_text(&self.session)?.as_ref());
        let turns: Vec<Turn<'_>> =
            log.spoken().iter().enumerate().filter_map(|(at, spoken)| Turn::kept(at, spoken, known.uuids())).collect();
        let excerpts = Excerpt::cut(&turns);
        files.write(&self.written, Bytes::default())?;
        let batches = excerpts
            .chunks(Pairs::FAN)
            .map(|batch| self.extracted(files, crew, batch))
            .collect::<Result<Vec<Extracted>, Failure>>()?;
        let extracted = Extracted {
            excerpts: excerpts.len(),
            pairs: batches.iter().map(|batch| batch.pairs).sum(),
            tokens: batches.iter().map(|batch| batch.tokens).sum(),
        };
        Ok(Uttered::new(Shown::said(extracted), Ok(Outcome::Paired)))
    }

    fn extracted<D: Dispatch, C: Clock>(
        &self,
        files: &impl Files,
        crew: &Crew<'_, D, C>,
        batch: &[Excerpt<'_>],
    ) -> Result<Extracted, Failure> {
        let prompts: Vec<Prompt> = batch.iter().map(Prompt::from).collect();
        let heard = crew.fanned(&prompts, Pairs::TIMEOUT)?;
        let pairs: BTreeSet<Pair> =
            batch.iter().zip(&heard).flat_map(|(excerpt, read)| excerpt.verified(read)).collect();
        let listed: Vec<Pair> = pairs.into_iter().collect();
        files.append(&self.written, &Bytes::from(&PairLines::from(listed.as_slice())))?;
        Ok(Extracted { excerpts: batch.len(), pairs: listed.len(), tokens: heard.iter().map(Heard::tokens).sum() })
    }
}

impl fmt::Display for Extracted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "excerpts\tpairs\ttokens\n{}\t{}\t{}", self.excerpts, self.pairs, self.tokens)
    }
}
