mod role;
use std::collections::BTreeMap;
use std::path::Path;

use syn::visit::Visit;
use syn::{ImplItemFn, ItemImpl, ItemMod, Receiver};

use super::declarations::Declarations;
use super::edge::{Edge, EdgeKey};
use super::edge_count::EdgeCount;
use super::finding::Finding;
use super::function_shape::{Body, FunctionShape, Returns};
use super::impl_context::{ImplContext, ImplKind};
use lint_core::TypeName;
use lint_core::Violation;
use lint_core::Walk;
use lint_core::{Bound, Site};
use lint_file::Concluded;
use lint_layout::{Length, Name};
use role::Role;

pub struct EdgeScan<'scan, 'file> {
    declarations: &'scan Declarations<'scan>,
    path: &'file Path,
    edges: BTreeMap<Edge<'scan>, BTreeMap<Name, Site<'file>>>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> EdgeScan<'scan, 'file> {
    pub fn new(declarations: &'scan Declarations<'scan>) -> Self {
        EdgeScan { declarations, path: Path::new(""), edges: BTreeMap::new(), violations: Vec::new() }
    }

    pub fn scan(&mut self, path: &'file Path, syntax: &syn::File) {
        self.path = path;
        self.visit_file(syntax);
    }

    pub fn into_edges(self) -> ScannedEdges<'file> {
        let edges = self.edges.into_iter().flat_map(|(edge, names)| {
            names.into_iter().map(move |(name, site)| EdgeRecord { key: EdgeKey::from(edge), name, site })
        });
        ScannedEdges { violations: self.violations, edges: edges.collect() }
    }

    fn check_function(&mut self, context: &ImplContext<'scan>, function: &ImplItemFn) {
        let shape = FunctionShape::describe(self.path, function, context, self.declarations);
        self.violations.extend(
            matches!(shape.returns(), Returns::TupleHoldingVertex)
                .then(|| Violation::new(shape.site(), Finding::TupleHoldsVertex)),
        );
        Role::classified(function.sig.receiver(), context, &shape).checked(self, context, function, &shape);
    }

    fn record_edge(
        &mut self,
        context: &ImplContext<'scan>,
        function: &ImplItemFn,
        shape: &FunctionShape<'scan, 'file>,
        edge: Edge<'scan>,
    ) {
        match context.public(function) {
            Some(public) => {
                self.edges.entry(edge).or_default().entry(context.edge_name(public)).or_insert_with(|| shape.site());
            }
            None => self.violations.push(Violation::new(shape.site(), Finding::HiddenEdge { edge })),
        }
    }

    fn view_fault(
        context: &ImplContext<'scan>,
        receiver: &Receiver,
        shape: &FunctionShape<'scan, 'file>,
    ) -> Option<Violation<'file>> {
        receiver.mutability.map_or_else(
            || {
                shape
                    .returns()
                    .vertex()
                    .filter(|_| shape.body() != Body::FieldProjection)
                    .map(|target| Violation::new(shape.site(), Finding::ViewReturnsVertex { target }))
            },
            |_| {
                matches!(context.kind(), ImplKind::InherentOnVertex(_))
                    .then(|| Violation::new(shape.site(), Finding::MutatesVertex { vertex: context.self_type() }))
            },
        )
    }
}

impl<'ast> Visit<'ast> for EdgeScan<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.scanned(item));
    }
}

impl EdgeScan<'_, '_> {
    fn scanned(&mut self, item: &ItemImpl) {
        let context = ImplContext::describe(item, self.declarations);
        item.items
            .iter()
            .filter_map(FunctionShape::function)
            .for_each(|function| self.check_function(&context, function));
        syn::visit::visit_item_impl(self, item);
    }
}

#[derive(Debug)]
pub struct EdgeRecord<'file> {
    key: EdgeKey,
    name: Name,
    site: Site<'file>,
}

#[derive(Debug, Default)]
pub struct ScannedEdges<'file> {
    violations: Vec<Violation<'file>>,
    edges: Vec<EdgeRecord<'file>>,
}

impl<'file> ScannedEdges<'file> {
    pub fn including(mut self, more: Vec<Violation<'file>>) -> ScannedEdges<'file> {
        self.violations.extend(more);
        self
    }
}

#[derive(Debug, Default)]
pub struct EdgeIndex<'file> {
    violations: Vec<Violation<'file>>,
    edges: BTreeMap<EdgeKey, BTreeMap<Name, Site<'file>>>,
}

impl<'file> FromIterator<ScannedEdges<'file>> for EdgeIndex<'file> {
    fn from_iter<Parts: IntoIterator<Item = ScannedEdges<'file>>>(parts: Parts) -> EdgeIndex<'file> {
        parts.into_iter().fold(EdgeIndex::default(), |mut index, part| {
            index.violations.extend(part.violations);
            part.edges.into_iter().for_each(|record| {
                index.edges.entry(record.key).or_default().entry(record.name).or_insert(record.site);
            });
            index
        })
    }
}

impl EdgeIndex<'_> {
    const EDGES_PER_VERTEX: Bound = Bound::Most(4);
}

impl<'file> From<EdgeIndex<'file>> for Concluded<'file> {
    fn from(index: EdgeIndex<'file>) -> Concluded<'file> {
        let most = EdgeIndex::EDGES_PER_VERTEX;
        let duplicates = index.edges.iter().filter(|(_, names)| names.len() > 1).filter_map(|(key, names)| {
            let site = *names.values().min()?;
            let edge = Edge::from(key);
            Some(Violation::new(site, Finding::DuplicateEdge { edge: &edge, sites: names }))
        });
        let fan_out = index.edges.iter().fold(
            BTreeMap::<&TypeName, BTreeMap<&Name, &Site<'file>>>::new(),
            |mut fan_out, (key, names)| {
                fan_out.entry(key.source()).or_default().extend(names.iter());
                fan_out
            },
        );
        let crowded = fan_out
            .into_iter()
            .filter(|(_, names)| most.allows(Length::new(names.len())).is_none())
            .filter_map(|(source, names)| {
                let site = **names.values().min()?;
                Some(Violation::new(site, Finding::CrowdedVertex { source, edges: EdgeCount::new(names.len()), most }))
            });
        let structural: Vec<Violation<'file>> = duplicates.chain(crowded).collect();
        index.violations.into_iter().chain(structural).collect()
    }
}
