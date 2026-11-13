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

use declarations::Declarations;
use declared_graphs::DeclaredGraphs;
use edge_scan::{EdgeIndex, EdgeScan, ScannedEdges};
use field_visibility::FieldVisibility;
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, PrimedLint, Reads};
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

impl PrimedLint for StateGraph {
    const HEADLINE: Headline = Headline::new("state graph violated");
    const READS: Reads = Reads::Shipped;

    type PrimerPart<'head> = Option<Declarations<'head>>;
    type Primer<'head> = Declarations<'head>;
    type Part<'head> = StateFile<'head>;
    type Context<'head> = Concluded<'head>;

    fn primed<'head>(parsed: Parsed<'_, 'head>) -> Option<Declarations<'head>> {
        parsed.file().shipped().map(|_| Declarations::from(parsed))
    }

    fn contributed<'head>(parsed: Parsed<'_, 'head>, declarations: &Declarations<'head>) -> StateFile<'head> {
        let path = parsed.head().path();
        let shipped = parsed.file().shipped().map(|code| {
            let mut edges = EdgeScan::new(declarations);
            edges.scan(path, code.syntax());
            let own = Vec::from(FieldVisibility::from(code))
                .into_iter()
                .chain(VariantMatches::violations(code, declarations))
                .chain(Vec::from(DeclaredGraphs::from(code)))
                .map(|violation| violation.moved(path));
            edges.into_edges().including(own.collect())
        });
        StateFile(shipped.unwrap_or_default())
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
