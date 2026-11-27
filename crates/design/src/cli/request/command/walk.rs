mod boundary;

use std::path::{Path, PathBuf};

use console_ports::{Environment, Shown};
use design_messages::Tally;
use dispatch_ports::{Clock, Dispatch, Prompt, Timeout};
use file_ports::Files;
use newtype::Wrapping;
use store_ports::Store;
use transcript_ports::Body;

use super::Ground;
use super::crew::{Crew, Heard};
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::{Choosing, Node};
pub use boundary::{Retrieved, Tree};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walk {
    tree: PathBuf,
}

#[derive(Debug)]
pub struct Walking<'ports, D: Dispatch, C: Clock> {
    crew: &'ports Crew<'ports, D, C>,
}

#[derive(Debug, Default)]
pub struct Walked<'tree> {
    reached: Vec<&'tree Node>,
    tokens: Tally,
}

#[derive(Debug)]
struct Level<'tree> {
    frontier: Vec<&'tree [Node]>,
    walked: Walked<'tree>,
}

impl From<PathBuf> for Walk {
    fn from(tree: PathBuf) -> Walk {
        Walk { tree }
    }
}

impl Walk {
    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock>(
        &self,
        environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        let tree = Walk::planted(ground.files(), &self.tree)?;
        let heard = environment.input().map_err(Failure::Deaf)?;
        let question = Body::try_new(heard.into_inner()).map_err(|_malformed| Failure::Unasked)?;
        let walked = Walking::new(crew).walked(&tree, &question)?;
        Ok(Uttered::new(Shown::said(walked.retrieved()), Ok(Outcome::Walked)))
    }

    pub fn planted(files: &impl Files, path: &Path) -> Result<Tree, Failure> {
        Tree::read(files.read_text(path)?.as_ref())
    }
}

impl<'ports, D: Dispatch, C: Clock> Walking<'ports, D, C> {
    const TIMEOUT: Timeout = Timeout::new(120);

    pub fn new(crew: &'ports Crew<'ports, D, C>) -> Walking<'ports, D, C> {
        Walking { crew }
    }

    pub fn walked<'tree>(&self, tree: &'tree Tree, question: &Body) -> Result<Walked<'tree>, Failure> {
        self.descended(question, Level { frontier: vec![tree.roots()], walked: Walked::default() })
    }

    fn descended<'tree>(&self, question: &Body, level: Level<'tree>) -> Result<Walked<'tree>, Failure> {
        let Some(_frontier) = level.frontier.first() else {
            return Ok(level.walked);
        };
        let choices: Vec<Choosing<'_, 'tree>> =
            level.frontier.iter().map(|children| Choosing::new(question, children)).collect();
        let prompts: Vec<Prompt> = choices.iter().map(Prompt::from).collect();
        let heard = self.crew.fanned(&prompts, Self::TIMEOUT)?;
        let chosen: Vec<&'tree Node> =
            choices.into_iter().zip(&heard).flat_map(|(choosing, read)| choosing.chosen(read)).collect();
        let frontier = chosen.iter().map(|node| node.children()).filter(|children| !children.is_empty());
        let next = Level { frontier: frontier.collect(), walked: level.walked.grown(chosen, &heard) };
        self.descended(question, next)
    }
}

impl<'tree> Walked<'tree> {
    fn grown(self, chosen: Vec<&'tree Node>, heard: &[Heard]) -> Walked<'tree> {
        let tokens: Tally = [self.tokens].into_iter().chain(heard.iter().map(Heard::tokens)).sum();
        Walked { reached: self.reached.into_iter().chain(chosen).collect(), tokens }
    }

    pub fn retrieved(&self) -> Retrieved<'tree> {
        Retrieved::reached(self.reached.iter().copied())
    }

    pub fn tokens(&self) -> Tally {
        self.tokens
    }
}
