mod declarations;
mod declared_graphs;
mod edge;
mod edge_count;
mod edge_scan;
mod field_visibility;
mod finding;
mod function_shape;
mod impl_context;
mod variant_matches;

use declarations::{DeclarationGathering, Declarations};
use declared_graphs::DeclaredGraphs;
use edge_scan::{EdgeIndex, EdgeScan, ScannedEdges};
use field_visibility::FieldVisibility;
use lint_core::{Headline, Home, Parsed, SourceFile};
use lint_file::{Concluded, PrimedFacts, PrimedLint, Reads};
use lint_sweep::{Gathers, Judgement, Watching};
use variant_matches::VariantMatches;

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
    type Contribution<'run, 'head: 'run> = StateContribution<'run, 'head>;

    fn priming<'head>(parsed: Parsed<'_, 'head>) -> DeclarationGathering<'head> {
        DeclarationGathering::from(parsed)
    }

    fn contribution<'run, 'head: 'run>(
        parsed: Parsed<'_, 'head>,
        declarations: &'run Declarations<'head>,
    ) -> StateContribution<'run, 'head> {
        let path = parsed.head().path();
        let watchers = parsed.file().shipped().map(|_| StateWatchers {
            edges: EdgeScan::new(path, declarations),
            fields: FieldVisibility::new(path),
            matches: VariantMatches::new(path, Home::from(parsed.head()), declarations),
            graphs: DeclaredGraphs::new(path),
        });
        StateContribution { watchers }
    }
}

#[derive(Debug)]
pub struct StateContribution<'run, 'head> {
    watchers: Option<StateWatchers<'run, 'head>>,
}

#[derive(Debug)]
struct StateWatchers<'run, 'head> {
    edges: EdgeScan<'run, 'head>,
    fields: FieldVisibility<'head>,
    matches: VariantMatches<'run, 'head>,
    graphs: DeclaredGraphs<'head>,
}

impl<'head> Gathers<StateFile<'head>> for StateContribution<'_, 'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.watchers.as_mut().map_or_default(|watchers| {
            let all: Vec<&mut Watching<'_>> =
                vec![&mut watchers.edges, &mut watchers.fields, &mut watchers.matches, &mut watchers.graphs];
            all
        })
    }

    fn gathered(self) -> StateFile<'head> {
        let scanned = self.watchers.map(|watchers| {
            let own = Vec::from(watchers.fields)
                .into_iter()
                .chain(watchers.matches.violations())
                .chain(Vec::from(watchers.graphs));
            watchers.edges.into_edges().including(own.collect())
        });
        StateFile(scanned.unwrap_or_default())
    }
}

impl PrimedLint for StateGraph {
    const HEADLINE: Headline = Headline::new("state graph violated");
    const READS: Reads = Reads::Shipped;

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        _: &'run Declarations<'head>,
        concluded: &'run Concluded<'head>,
    ) -> Judgement<'file, 'run> {
        Judgement::found(concluded.found_in(file))
    }
}
