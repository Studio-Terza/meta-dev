mod declarations;
mod declared_graphs;
mod edge;
mod edge_count;
mod edge_scan;
mod field_visibility;
mod finding;
mod function_shape;
mod impl_context;
mod receiving;
mod variant_matches;

use declarations::{DeclarationGathering, Declarations};
use declared_graphs::DeclaredGraphs;
use edge_scan::{EdgeIndex, EdgeScan, ScannedEdges, Written};
use field_visibility::FieldVisibility;
use lint_core::{Headline, Home, Parsed, SourceFile, Violation};
use lint_file::{Concluded, FileFacts, PrimedFacts, PrimedLint, Reads};
use lint_sweep::{Gathers, Watching};
use variant_matches::{MatchCandidates, VariantMatches};

#[derive(Debug)]
pub struct StateGraph;

#[derive(Debug)]
pub struct StateFile<'head>(ScannedEdges<'head>);

impl<'head> FromIterator<StateFile<'head>> for Concluded<'head> {
    fn from_iter<Parts: IntoIterator<Item = StateFile<'head>>>(parts: Parts) -> Concluded<'head> {
        Concluded::from(parts.into_iter().map(|part| part.0).collect::<EdgeIndex<'head>>())
    }
}

impl PrimedFacts for StateGraph {
    type PrimerPart<'head> = Option<Declarations<'head>>;
    type Primer<'head> = Declarations<'head>;
    type Part<'head> = StateFile<'head>;
    type Context<'head> = Concluded<'head>;

    type Priming<'head> = DeclarationGathering<'head>;
    type Raw<'head> = StateRaw<'head>;
    type Recording<'run, 'head: 'run> = StateRecording<'run, 'head>;

    fn priming<'head>(parsed: Parsed<'_, 'head>) -> DeclarationGathering<'head> {
        DeclarationGathering::from(parsed)
    }

    fn primer_completed<'head>(
        part: Option<Declarations<'head>>,
        facts: &FileFacts<'head>,
    ) -> Option<Declarations<'head>> {
        part.map(|declarations| declarations.homed(facts.as_ref()))
    }

    fn recording<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> StateRecording<'run, 'head> {
        let path = parsed.head().path();
        let watchers = parsed.file().shipped().map(|_| StateWatchers {
            edges: EdgeScan::from(path),
            fields: FieldVisibility::new(path),
            matches: VariantMatches::new(path, Home::from(parsed.head())),
            graphs: DeclaredGraphs::new(parsed.file(), path),
        });
        StateRecording { watchers }
    }

    fn contribution<'head>(raw: Self::Raw<'head>, declarations: &Self::Primer<'head>) -> Self::Part<'head> {
        StateFile(raw.0.map_or_default(|recorded| {
            let own = recorded.own.into_iter().chain(recorded.matches.foreign(declarations));
            recorded.edges.resolved(declarations).including(own.collect())
        }))
    }
}

#[derive(Debug)]
pub struct StateRecording<'run, 'head> {
    watchers: Option<StateWatchers<'run, 'head>>,
}

#[derive(Debug)]
struct StateWatchers<'run, 'head> {
    edges: EdgeScan<'head>,
    fields: FieldVisibility<'head>,
    matches: VariantMatches<'head>,
    graphs: DeclaredGraphs<'run, 'head>,
}

#[derive(Debug)]
pub struct StateRaw<'head>(Option<RecordedState<'head>>);

#[derive(Debug)]
struct RecordedState<'head> {
    edges: Written<'head>,
    matches: MatchCandidates<'head>,
    own: Vec<Violation<'head>>,
}

impl<'head> Gathers<StateRaw<'head>> for StateRecording<'_, 'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.watchers.as_mut().map_or_default(|watchers| {
            let all: Vec<&mut Watching<'_>> =
                vec![&mut watchers.edges, &mut watchers.fields, &mut watchers.matches, &mut watchers.graphs];
            all
        })
    }

    fn gathered(self) -> StateRaw<'head> {
        StateRaw(self.watchers.map(|watchers| RecordedState {
            edges: watchers.edges.gathered(),
            matches: watchers.matches.gathered(),
            own: Vec::from(watchers.fields).into_iter().chain(Vec::from(watchers.graphs)).collect(),
        }))
    }
}

impl PrimedLint for StateGraph {
    const HEADLINE: Headline = Headline::new("state graph violated");
    const READS: Reads = Reads::Shipped;

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        _: &Declarations<'head>,
        concluded: &Concluded<'head>,
    ) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
