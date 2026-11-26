use std::collections::BTreeSet;
use std::fmt;
use std::ops::Not;
use std::path::{Path, PathBuf};

use design_model::{Consideration, CrateName, FeatureName, Verdict};
use design_store::{
    Evidence, Feature, Ideal, IdealId, Judged, Judgment, LineNumber, Quote, Revision, Ruling, Said, SourcePath,
    Unabridged,
};
use dispatch_ports::Reply;
use file_ports::{FileFailure, Files, Kind, Text};
use grammar_git::{GitFile, Object, RefName};
use invariants::Ignoring;
use newtype::Wrapping;
use serde::Deserialize;
use text_lines::Lines;
use transcript_ports::{Body, Instant, Uuid};

use super::super::hook::answers::Replied;
use crate::cli::failure::Failure;
use crate::cli::output::Count;
use crate::cli::request::ignored::Ignored;
use crate::cli::request::projects::Projects;
use crate::{Outcome, Uttered};
use console_ports::Shown;

#[derive(Debug, Deserialize)]
struct Picked {
    uuid: Uuid,
    quote: Body,
}

#[derive(Debug, Deserialize)]
struct Drafted {
    quotes: Vec<Picked>,
}

#[derive(Debug, Deserialize)]
pub struct Grouping {
    ideals: Vec<usize>,
    #[serde(default)]
    supersedes: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Proposed {
    name: FeatureName,
    consideration: Consideration,
    quotes: Vec<Picked>,
    #[serde(default)]
    crates: Vec<CrateName>,
}

#[derive(Debug, Deserialize)]
struct Ruled {
    ideal: i64,
    verdict: Verdict,
    #[serde(default)]
    evidence: Vec<Pointed>,
}

#[derive(Debug, Deserialize)]
struct Pointed {
    file: SourcePath,
    line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate(Vec<Quote>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    Grouped(usize),
    Alone(usize),
}

#[derive(Debug)]
struct Placed {
    key: Key,
    id: IdealId,
}

#[derive(Debug)]
struct Keyed<'candidates> {
    candidates: &'candidates [Candidate],
    keys: Vec<Key>,
}

#[derive(Debug)]
struct Firsts<'items, Item>(&'items [Item]);

#[derive(Debug)]
struct Quoted<'quotes>(&'quotes [Quote]);

#[derive(Debug)]
pub struct Merging<'candidates> {
    candidates: &'candidates [Candidate],
    groupings: Vec<Grouping>,
}

#[derive(Debug)]
pub struct Extracting<'history>(Vec<Unabridged<'history>>);

#[derive(Debug)]
pub struct Consolidating<'candidates>(&'candidates [Candidate]);

#[derive(Debug)]
pub struct Proposing<'history> {
    members: &'history [CrateName],
    shown: Vec<Unabridged<'history>>,
}

#[derive(Debug)]
pub struct Judging<'judged> {
    ideals: &'judged [Ideal],
    sources: &'judged Sources,
}

#[derive(Debug)]
struct Source {
    path: SourcePath,
    lines: Vec<String>,
}

#[derive(Debug)]
pub struct Sources(Vec<Source>);

#[derive(Debug)]
pub struct Workspace<'ground, F: Files> {
    files: &'ground F,
    root: &'ground Path,
}

#[derive(Debug)]
struct Weighed {
    verdict: Verdict,
    rulings: Vec<Ruling>,
}

#[derive(Debug)]
pub struct Verdicts {
    rulings: Vec<Ruling>,
    findings: Count,
}

#[derive(Debug)]
pub struct Findings {
    judgments: Vec<Judgment>,
    findings: Count,
}

#[derive(Debug)]
pub struct Table<'rows, Row> {
    header: &'static str,
    rows: &'rows [Row],
}

impl<Item: PartialEq + Clone> Firsts<'_, Item> {
    fn kept(&self) -> Vec<Item> {
        self.0
            .iter()
            .enumerate()
            .filter(|(index, item)| self.0.get(..*index).unwrap_or_default().contains(item).not())
            .map(|(_index, item)| item.clone())
            .collect()
    }
}

impl Picked {
    fn quoted(&self, chunk: &[Said]) -> Option<Quote> {
        chunk.iter().find_map(|said| said.quoted(&self.uuid, &self.quote))
    }

    fn kept(picked: &[Picked], chunk: &[Said]) -> Vec<Quote> {
        let quoted: Vec<Quote> = picked.iter().filter_map(|pick| pick.quoted(chunk)).collect();
        Firsts(&quoted).kept()
    }
}

impl Candidate {
    pub fn read(reply: &Reply, chunk: &[Said]) -> Vec<Candidate> {
        Replied::from(reply)
            .listed::<Drafted>()
            .iter()
            .map(|drafted| Picked::kept(&drafted.quotes, chunk))
            .filter(|quotes| quotes.is_empty().not())
            .map(Candidate)
            .collect()
    }
}

impl Grouping {
    pub fn read(reply: &Reply) -> Vec<Grouping> {
        Replied::from(reply).listed()
    }
}

impl<'candidates> Merging<'candidates> {
    pub fn new(candidates: &'candidates [Candidate], groupings: Vec<Grouping>) -> Merging<'candidates> {
        Merging { candidates, groupings }
    }

    pub fn ideals(&self) -> Vec<Ideal> {
        let keyed = Keyed {
            candidates: self.candidates,
            keys: (1..=self.candidates.len()).map(|number| self.key(number)).collect(),
        };
        let placed: Vec<Placed> = Firsts(&keyed.keys)
            .kept()
            .into_iter()
            .zip((1_i64..).map(IdealId::new))
            .map(|(key, id)| Placed { key, id })
            .collect();
        placed
            .iter()
            .map(|place| {
                let ideal = Ideal::new(place.id, keyed.quotes(place.key));
                self.later(&keyed, &placed, place.key)
                    .map_or_else(|| ideal.clone(), |later| ideal.clone().superseded_by(later))
            })
            .collect()
    }

    fn later(&self, keyed: &Keyed<'_>, placed: &[Placed], earlier: Key) -> Option<IdealId> {
        let superseding = self.superseding(keyed, earlier)?;
        placed.iter().find(|place| place.key == superseding).map(|place| place.id)
    }

    fn key(&self, number: usize) -> Key {
        self.groupings
            .iter()
            .position(|grouping| grouping.ideals.contains(&number))
            .map_or(Key::Alone(number), Key::Grouped)
    }

    fn superseding(&self, keyed: &Keyed<'_>, earlier: Key) -> Option<Key> {
        let before = keyed.latest(earlier);
        self.groupings
            .iter()
            .enumerate()
            .filter(|(_index, grouping)| grouping.supersedes.iter().any(|number| self.key(*number) == earlier))
            .map(|(index, _grouping)| Key::Grouped(index))
            .filter(|later| *later != earlier)
            .find(|later| keyed.latest(*later) > before)
    }
}

impl Keyed<'_> {
    fn quotes(&self, key: Key) -> Vec<Quote> {
        let gathered: Vec<Quote> = self
            .candidates
            .iter()
            .zip(&self.keys)
            .filter(|(_candidate, each)| **each == key)
            .flat_map(|(candidate, _each)| candidate.0.iter().cloned())
            .collect();
        Firsts(&gathered).kept()
    }

    fn latest(&self, key: Key) -> Option<Instant> {
        self.quotes(key).iter().map(|quote| AsRef::<Instant>::as_ref(quote).clone()).max()
    }
}

impl<'history> From<&'history [Said]> for Extracting<'history> {
    fn from(chunk: &'history [Said]) -> Extracting<'history> {
        Extracting(chunk.iter().map(Unabridged::from).collect())
    }
}

impl<'candidates> From<&'candidates [Candidate]> for Consolidating<'candidates> {
    fn from(candidates: &'candidates [Candidate]) -> Consolidating<'candidates> {
        Consolidating(candidates)
    }
}

impl<'history> Proposing<'history> {
    pub fn new(members: &'history [CrateName], chunk: &'history [Said]) -> Proposing<'history> {
        Proposing { members, shown: chunk.iter().map(Unabridged::from).collect() }
    }
}

impl Proposed {
    pub fn read(reply: &Reply, chunk: &[Said], members: &[CrateName]) -> Vec<Feature> {
        Replied::from(reply)
            .listed::<Proposed>()
            .iter()
            .filter_map(|proposed| proposed.feature(chunk, members))
            .collect()
    }

    pub fn merged(proposed: &[Feature]) -> Vec<Feature> {
        let names: Vec<&FeatureName> = proposed.iter().map(AsRef::<FeatureName>::as_ref).collect();
        Firsts(&names)
            .kept()
            .into_iter()
            .filter_map(|name| {
                proposed
                    .iter()
                    .filter(|feature| AsRef::<FeatureName>::as_ref(*feature) == name)
                    .cloned()
                    .reduce(Feature::absorbed)
            })
            .collect()
    }

    fn feature(&self, chunk: &[Said], members: &[CrateName]) -> Option<Feature> {
        let quotes = Picked::kept(&self.quotes, chunk);
        let crates: Vec<CrateName> = self.crates.iter().filter(|named| members.contains(named)).cloned().collect();
        quotes.first().map(|_first| Feature::new(self.name.clone(), self.consideration, quotes.clone(), crates))
    }
}

impl Ruled {
    fn weighed(found: Option<&Ruled>, ideal: IdealId, sources: &Sources) -> Weighed {
        let cited: Vec<Evidence> =
            found.map_or_default(|ruled| ruled.evidence.iter().filter_map(|pointed| sources.cited(pointed)).collect());
        let claimed = found.map_or(Verdict::Unclear, |ruled| ruled.verdict);
        let verdict = cited.first().map_or_else(|| claimed.unproven(), |_first| claimed);
        let rulings = cited.first().map_or_else(
            || vec![Ruling::new(ideal, verdict, None)],
            |_first| cited.iter().map(|evidence| Ruling::new(ideal, verdict, Some(evidence.clone()))).collect(),
        );
        Weighed { verdict, rulings }
    }
}

impl Verdicts {
    pub fn read(reply: Option<&Reply>, ideals: &[Ideal], sources: &Sources) -> Verdicts {
        let ruled: Vec<Ruled> = reply.map_or_default(|replied| Replied::from(replied).listed());
        let weighed: Vec<Weighed> = ideals
            .iter()
            .map(|ideal| {
                let id = *AsRef::<IdealId>::as_ref(ideal);
                Ruled::weighed(ruled.iter().find(|each| each.ideal == id.into_inner()), id, sources)
            })
            .collect();
        let findings = Count::new(weighed.iter().filter(|each| each.verdict != Verdict::Holds).count());
        Verdicts { rulings: weighed.into_iter().flat_map(|each| each.rulings).collect(), findings }
    }

    pub fn judged(self, feature: &FeatureName, revision: &Revision) -> Findings {
        let judgments =
            self.rulings.into_iter().map(|ruling| ruling.judged(feature.clone(), revision.clone())).collect();
        Findings { judgments, findings: self.findings }
    }
}

impl From<&Findings> for Judged {
    fn from(findings: &Findings) -> Judged {
        Judged::from(findings.judgments.clone())
    }
}

impl<'judged> Judging<'judged> {
    pub fn new(ideals: &'judged [Ideal], sources: &'judged Sources) -> Judging<'judged> {
        Judging { ideals, sources }
    }
}

impl Source {
    fn read(root: &Path, path: &Path, text: &Text) -> Option<Source> {
        let relative = path.strip_prefix(root).ignoring(Ignored::Unsourced)?.to_str()?.to_owned();
        let lines: Vec<String> = AsRef::<str>::as_ref(text).parse::<Lines>().into_iter().flatten().collect();
        SourcePath::try_new(relative).ignoring(Ignored::Unsourced).map(|named| Source { path: named, lines })
    }
}

impl Sources {
    pub fn present(&self) -> Option<&Sources> {
        self.0.first().map(|_first| self)
    }

    fn cited(&self, pointed: &Pointed) -> Option<Evidence> {
        self.0
            .iter()
            .find(|source| source.path == pointed.file)
            .filter(|source| source.lines.iter().zip(1_u32..).any(|(_line, number)| number == pointed.line))
            .map(|source| Evidence::new(source.path.clone(), LineNumber::new(pointed.line)))
    }
}

impl<'ground, F: Files> Workspace<'ground, F> {
    const CRATES: &'static str = "crates";
    const MANIFEST: &'static str = "Cargo.toml";
    const SOURCE: &'static str = "src";
    const RUST: &'static str = "rs";
    const GIT: &'static str = ".git";
    const HEAD: &'static str = "HEAD";
    const COMMON: &'static str = "commondir";
    const PACKED: &'static str = "packed-refs";

    pub fn new(files: &'ground F, root: &'ground Path) -> Workspace<'ground, F> {
        Workspace { files, root }
    }

    pub fn members(&self) -> Vec<CrateName> {
        let directories =
            self.files.entries(&self.root.join(Self::CRATES)).ignoring(Ignored::Uncrated).unwrap_or_default();
        let manifests: Vec<PathBuf> = directories.iter().map(|directory| directory.join(Self::MANIFEST)).collect();
        let named: BTreeSet<CrateName> = directories
            .iter()
            .zip(self.files.kinds(&manifests))
            .filter(|(_directory, kind)| *kind == Kind::File)
            .filter_map(|(directory, _kind)| directory.file_name().and_then(|name| name.to_str()))
            .filter_map(|name| CrateName::try_new(name.to_owned()).ignoring(Ignored::Uncrated))
            .collect();
        named.into_iter().collect()
    }

    pub fn sources(&self, crates: &[CrateName]) -> Result<Sources, FileFailure> {
        let roots: Vec<PathBuf> =
            crates.iter().map(|member| self.root.join(Self::CRATES).join(member.as_ref()).join(Self::SOURCE)).collect();
        let paths =
            roots.iter().map(|root| self.walked(root)).collect::<Result<Vec<Vec<PathBuf>>, FileFailure>>()?.concat();
        let texts = self.files.read_text_many(&paths).into_iter().collect::<Result<Vec<Text>, FileFailure>>()?;
        Ok(Sources(paths.iter().zip(&texts).filter_map(|(path, text)| Source::read(self.root, path, text)).collect()))
    }

    pub fn revision(&self) -> Result<Revision, Failure> {
        let dot = self.root.join(Self::GIT);
        let linked = self.read(&dot).linked().map(|path| self.root.join(path.as_ref()));
        let gitdir = Projects::resolved(&linked.unwrap_or(dot));
        let shared = self.read(&gitdir.join(Self::COMMON)).common().map(|path| gitdir.join(path.as_ref()));
        let common = Projects::resolved(&shared.unwrap_or_else(|| gitdir.clone()));
        let head = self.read(&gitdir.join(Self::HEAD));
        let object =
            head.object().or_else(|| head.symbolic().and_then(|name| self.resolved(&[&gitdir, &common], &name)));
        object
            .and_then(|found| Revision::try_new(found.as_ref().to_owned()).ignoring(Ignored::Unrevised))
            .ok_or_else(|| Failure::Unrevised(self.root.to_path_buf()))
    }

    fn resolved(&self, directories: &[&PathBuf; 2], name: &RefName) -> Option<Object> {
        let [gitdir, common] = directories;
        [gitdir.join(name.as_ref()), common.join(name.as_ref())]
            .iter()
            .find_map(|path| self.read(path).object())
            .or_else(|| self.read(&common.join(Self::PACKED)).packed(name))
    }

    fn read(&self, path: &Path) -> GitFile {
        let text = self.files.read_text(path).ignoring(Ignored::Unrevised).unwrap_or_default();
        GitFile::read(text.as_ref())
    }

    fn walked(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        let entries: Vec<PathBuf> =
            self.files.entries(directory)?.into_iter().collect::<BTreeSet<PathBuf>>().into_iter().collect();
        let kinds = self.files.kinds(&entries);
        let nested = entries
            .iter()
            .zip(&kinds)
            .filter(|(_entry, kind)| **kind == Kind::Directory)
            .map(|(entry, _kind)| self.walked(entry))
            .collect::<Result<Vec<Vec<PathBuf>>, FileFailure>>()?;
        let rust = entries
            .iter()
            .zip(&kinds)
            .filter(|(_entry, kind)| **kind == Kind::File)
            .filter(|(entry, _kind)| entry.extension().and_then(|found| found.to_str()) == Some(Self::RUST))
            .map(|(entry, _kind)| entry.clone());
        Ok(rust.chain(nested.concat()).collect())
    }
}

impl<'rows, Row> Table<'rows, Row> {
    pub const IDEALS: &'static str = "ideal\tstanding\tmessage\texcerpt";
    pub const FEATURES: &'static str = "feature\tconsideration\tcrates\tmessage\texcerpt";
    pub const JUDGMENTS: &'static str = "feature\tideal\tverdict\tevidence";

    pub fn new(header: &'static str, rows: &'rows [Row]) -> Table<'rows, Row> {
        Table { header, rows }
    }
}

impl fmt::Display for Extracting<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Below are messages from the owner of a software project, each headed by [uuid timestamp].\n\
             List every ideal the owner holds about how the project's code, checks or process must be: a rule, principle, preference or prohibition.\n\
             State each ideal only in the owner's own words. Never paraphrase or summarise: give one or more excerpts that state it, each copied exactly, character for character, one or two sentences, from the message it is in.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"quotes\": [{{\"uuid\": the full uuid from the header, \"quote\": the excerpt}}]}}, one entry per ideal. Reply [] if there are none.\n\n{}",
            self.0.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n\n")
        )
    }
}

impl fmt::Display for Quoted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|quote| write!(formatter, "\n   {quote}"))
    }
}

impl fmt::Display for Consolidating<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let numbered: Vec<String> = self
            .0
            .iter()
            .zip(1_usize..)
            .map(|(candidate, number)| format!("{number}.{}", Quoted(&candidate.0)))
            .collect();
        write!(
            formatter,
            "Below, numbered, are ideals the owner of a software project holds, each given only as the owner's own words with the message uuid and timestamp.\n\
             Group the ideals that state the same rule. Where a later ideal contradicts an earlier one, the later one supersedes it.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"ideals\": [the numbers of the ideals that state one rule], \"supersedes\": [the numbers of earlier ideals this rule contradicts]}}. A number belongs to at most one group. Reply [] if no two ideals state the same rule and none contradicts another.\n\n{}",
            numbered.join("\n")
        )
    }
}

impl fmt::Display for Proposing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let members: Vec<&str> = self.members.iter().map(AsRef::<str>::as_ref).collect();
        write!(
            formatter,
            "Below are the crates of a Rust workspace and messages from the project's owner, each headed by [uuid timestamp].\n\
             The project has three considerations: checks (code quality checks run in CI, such as lints, tests and budgets), process (the work around the code, such as stop hooks, the transcript database and agent orchestration) and shipping (publishing and installing the first two).\n\
             List the features the owner has asked for. Name each with a short kebab-case name, such as stop-hook; give its consideration; quote the owner's words that ask for it, each copied exactly, character for character, one or two sentences, from the message it is in; and name the crates whose code implements it, from the crates below only.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"name\": the feature's name, \"consideration\": \"checks\", \"process\" or \"shipping\", \"quotes\": [{{\"uuid\": the full uuid from the header, \"quote\": the excerpt}}], \"crates\": [crate names]}}. Reply [] if there are none.\n\n\
             Crates: {}\n\nMessages:\n\n{}",
            members.join(", "),
            self.shown.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n\n")
        )
    }
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let numbered: Vec<String> =
            self.lines.iter().zip(1_u32..).map(|(line, number)| format!("{number}: {line}")).collect();
        write!(formatter, "== {} ==\n{}", self.path, numbered.join("\n"))
    }
}

impl fmt::Display for Judging<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ideals: Vec<String> = self
            .ideals
            .iter()
            .map(|ideal| format!("i{}:{}", AsRef::<IdealId>::as_ref(ideal), Quoted(ideal.quotes())))
            .collect();
        let sources: Vec<String> = self.sources.0.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "Below are the ideals the owner of a software project holds, each headed i<number> and given only in the owner's own words, then every source file of one feature, each headed by its path, with its lines numbered.\n\
             For every ideal, judge this code against it: holds if the code meets it, violated if some code breaks it, unclear if this code cannot tell.\n\
             For violated, cite every line that breaks it; for holds or unclear, cite the lines the verdict rests on, if any.\n\
             Reply with JSON only, no prose and no code fence: a list of {{\"ideal\": the ideal's number, \"verdict\": \"holds\", \"violated\" or \"unclear\", \"evidence\": [{{\"file\": the path exactly as headed, \"line\": the line number}}]}}, one entry per ideal.\n\n\
             Ideals:\n{}\n\nSources:\n\n{}",
            ideals.join("\n"),
            sources.join("\n\n")
        )
    }
}

impl From<Vec<Ideal>> for Uttered {
    fn from(ideals: Vec<Ideal>) -> Uttered {
        let counted = Count::new(ideals.len());
        Uttered::new(
            Shown::said(Table::new(Table::<Ideal>::IDEALS, &ideals)),
            Ok(Outcome::Idealised { ideals: counted }),
        )
    }
}

impl From<Vec<Feature>> for Uttered {
    fn from(features: Vec<Feature>) -> Uttered {
        let counted = Count::new(features.len());
        Uttered::new(
            Shown::said(Table::new(Table::<Feature>::FEATURES, &features)),
            Ok(Outcome::Featured { features: counted }),
        )
    }
}

impl From<Findings> for Uttered {
    fn from(findings: Findings) -> Uttered {
        let counted = findings.findings;
        Uttered::new(Shown::said(findings), Ok(Outcome::Judged { findings: counted }))
    }
}

impl fmt::Display for Findings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", Table::new(Table::<Judgment>::JUDGMENTS, &self.judgments))
    }
}

impl From<Vec<Judgment>> for Uttered {
    fn from(judgments: Vec<Judgment>) -> Uttered {
        Uttered::new(Shown::said(Table::new(Table::<Judgment>::JUDGMENTS, &judgments)), Ok(Outcome::Judgments))
    }
}

impl<Row: fmt::Display> fmt::Display for Table<'_, Row> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", self.header)?;
        self.rows.iter().try_for_each(|row| writeln!(formatter, "{row}"))
    }
}
