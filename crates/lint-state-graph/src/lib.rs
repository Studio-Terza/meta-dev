use std::path::Path;
mod declarations;
mod declared_graphs;
mod edge;
mod edge_count;
mod edge_scan;
mod field_visibility;
mod finding;
mod foreign_matches;
mod function_shape;
mod impl_context;
mod receiving;

use declarations::{DeclarationGathering, Declarations};
use declared_graphs::DeclaredGraphs;
use edge_scan::{EdgeIndex, EdgeScan, ScannedEdges, Written};
use field_visibility::FieldVisibility;
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, PrimedFacts, PrimedLint, Reads};
use lint_sweep::{Contributing, Gathers};

pub use foreign_matches::ForeignMatches;

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
    type PrimerPart<'head> = Option<Declarations>;
    type Primer<'head> = Declarations;
    type Part<'head> = StateFile<'head>;
    type Context<'head> = Concluded<'head>;

    type Priming<'head> = DeclarationGathering;
    type Raw<'head> = StateRaw<'head>;
    type Recording<'run, 'head: 'run> = StateRecording<'run, 'head>;

    fn priming(parsed: Parsed<'_, '_>) -> DeclarationGathering {
        DeclarationGathering::from(parsed)
    }

    fn recording<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> StateRecording<'run, 'head> {
        let path = AsRef::<Path>::as_ref(parsed.head());
        let watchers = <&SourceFile>::from(parsed).shipped().map(|_| StateWatchers {
            edges: EdgeScan::from(path),
            fields: Contributing::new(FieldVisibility::from(path)),
            graphs: Contributing::new(DeclaredGraphs::new(<&SourceFile>::from(parsed), path)),
        });
        StateRecording { watchers }
    }

    fn contribution<'head>(raw: Self::Raw<'head>, declarations: &Self::Primer<'head>) -> Self::Part<'head> {
        StateFile(raw.0.map_or_default(|recorded| recorded.edges.resolved(declarations).including(recorded.own)))
    }
}

#[derive(Debug)]
pub struct StateRecording<'run, 'head> {
    watchers: Option<StateWatchers<'run, 'head>>,
}

#[derive(Debug)]
struct StateWatchers<'run, 'head> {
    edges: EdgeScan<'head>,
    fields: Contributing<FieldVisibility<'head>>,
    graphs: Contributing<DeclaredGraphs<'run, 'head>>,
}

#[derive(Debug)]
pub struct StateRaw<'head>(Option<RecordedState<'head>>);

#[derive(Debug)]
struct RecordedState<'head> {
    edges: Written<'head>,
    own: Vec<Violation<'head>>,
}

impl<'head> Gathers<StateRaw<'head>> for StateRecording<'_, 'head> {
    fn gathered(self, file: &SourceFile) -> StateRaw<'head> {
        StateRaw(self.watchers.map(|watchers| {
            RecordedState {
                edges: watchers.edges.gathered(file),
                own: watchers
                    .fields
                    .concluded(file, |_, (), found| found)
                    .into_iter()
                    .chain(watchers.graphs.concluded(file, |graphs, (), invoked| graphs.found(invoked)))
                    .collect(),
            }
        }))
    }
}

impl PrimedLint for StateGraph {
    const HEADLINE: Headline = Headline::new("state graph violated");
    const READS: Reads = Reads::Shipped;

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        _: &Declarations,
        concluded: &Concluded<'head>,
    ) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
