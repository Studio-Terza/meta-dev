mod boundary;

use design_model::FeatureName;
use design_store::{Feature, FeatureNamed, Featured, Ideal, Idealised, Ideals, Judged, OwnerHistory, Said};
use dispatch_ports::{Clock, Dispatch, Model, Prompt, Timeout};
use file_ports::Files;
use store_ports::Store;

use super::Ground;
use super::structure::{Crew, Heard};
use crate::cli::failure::Failure;
pub use boundary::Findings;
use boundary::{
    Candidate, Consolidating, Extracting, Grouping, Judging, Merging, Proposed, Proposing, Verdicts, Workspace,
};

#[derive(Debug)]
pub struct Verifier<'ground, S: Store, F: Files, D: Dispatch, C: Clock> {
    store: &'ground S,
    workspace: Workspace<'ground, F>,
    crew: &'ground Crew<'ground, D, C>,
}

impl<'ground, S: Store, F: Files, D: Dispatch, C: Clock> Verifier<'ground, S, F, D, C> {
    const CHUNK: usize = 300;
    const READING: Timeout = Timeout::new(300);
    const JUDGING: Timeout = Timeout::new(900);

    pub fn new(
        ground: &'ground Ground<'ground, F, S>,
        crew: &'ground Crew<'ground, D, C>,
    ) -> Verifier<'ground, S, F, D, C> {
        Verifier { store: ground.store(), workspace: Workspace::new(ground.files(), ground.located().repo()), crew }
    }

    pub fn idealised(&self) -> Result<Vec<Ideal>, Failure> {
        let history: Vec<Said> = self.store.read(OwnerHistory)?;
        let chunks: Vec<&[Said]> = history.chunks(Self::CHUNK).collect();
        let prompts: Vec<Prompt> = chunks.iter().map(|chunk| Prompt::from(&Extracting::from(*chunk))).collect();
        let heard = self.crew.fanned(&prompts, Self::READING, Model::Haiku)?;
        let candidates: Vec<Candidate> = chunks
            .iter()
            .zip(&heard)
            .flat_map(|(chunk, read)| read.reply().map_or_default(|reply| Candidate::read(reply, chunk)))
            .collect();
        let consolidated = candidates
            .first()
            .map(|_first| {
                let prompt = Prompt::from(&Consolidating::from(candidates.as_slice()));
                self.crew.replied(&prompt, Self::READING, Model::Haiku)
            })
            .transpose()?;
        let groupings = consolidated.as_ref().and_then(Heard::reply).map_or_default(Grouping::read);
        let ideals = Merging::new(&candidates, groupings).ideals();
        self.store.write(Idealised::from(ideals.clone()))?;
        Ok(ideals)
    }

    pub fn featured(&self) -> Result<Vec<Feature>, Failure> {
        let members = self.workspace.members();
        let history: Vec<Said> = self.store.read(OwnerHistory)?;
        let chunks: Vec<&[Said]> = history.chunks(Self::CHUNK).collect();
        let prompts: Vec<Prompt> = chunks.iter().map(|chunk| Prompt::from(&Proposing::new(&members, chunk))).collect();
        let heard = self.crew.fanned(&prompts, Self::READING, Model::Sonnet)?;
        let proposed: Vec<Feature> = chunks
            .iter()
            .zip(&heard)
            .flat_map(|(chunk, read)| read.reply().map_or_default(|reply| Proposed::read(reply, chunk, &members)))
            .collect();
        let features = Proposed::merged(&proposed);
        self.store.write(Featured::from(features.clone()))?;
        Ok(features)
    }

    pub fn judged(&self, name: &FeatureName) -> Result<Findings, Failure> {
        let feature =
            self.store.read(FeatureNamed::from(name.clone()))?.ok_or_else(|| Failure::Unfeatured(name.clone()))?;
        let ideals: Vec<Ideal> =
            self.store.read(Ideals)?.into_iter().filter(|ideal| ideal.superseded().is_none()).collect();
        ideals.first().ok_or(Failure::Unidealised)?;
        let read = self.workspace.sources(feature.crates())?;
        let sources = read.present().ok_or_else(|| Failure::Unsourced(name.clone()))?;
        let revision = self.workspace.revision()?;
        let heard = self.crew.replied(&Prompt::from(&Judging::new(&ideals, sources)), Self::JUDGING, Model::Sonnet)?;
        let findings = Verdicts::read(heard.reply(), &ideals, sources).judged(name, &revision);
        self.store.write(Judged::from(&findings))?;
        Ok(findings)
    }
}
