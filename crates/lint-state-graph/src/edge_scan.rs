mod edge;
mod edge_count;

pub use edge::Edge;
pub use edge_count::EdgeCount;

use std::collections::BTreeMap;
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, Receiver};

use super::StateGraph;
use super::declarations::Declarations;
use super::finding::Finding;
use super::function_shape::{Body, FunctionShape, Returns};
use super::impl_context::{ImplContext, ImplKind};
use lint_core::Name;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::TypeName;
use lint_core::Violation;
use lint_core::Walk;

pub struct EdgeScan<'scan> {
    declarations: &'scan Declarations,
    path: PathBuf,
    edges: BTreeMap<Edge, BTreeMap<Name, Site>>,
    violations: Vec<Violation>,
}

impl<'scan> EdgeScan<'scan> {
    pub fn new(declarations: &'scan Declarations) -> Self {
        EdgeScan { declarations, path: PathBuf::new(), edges: BTreeMap::new(), violations: Vec::new() }
    }

    pub fn scan(&mut self, file: &SourceFile) {
        file.path().clone_into(&mut self.path);
        self.visit_file(file.syntax());
    }

    pub fn violations(mut self) -> Vec<Violation> {
        let most = StateGraph::EDGES_PER_VERTEX;
        let duplicates = self.edges.iter().filter(|(_, names)| names.len() > 1).filter_map(|(edge, names)| {
            let site = names.values().min()?.clone();
            Some(Violation::new(site, Finding::DuplicateEdge { edge: edge.clone(), sites: names.clone() }))
        });
        let fan_out = self.edges.iter().fold(
            BTreeMap::<&TypeName, BTreeMap<&Name, &Site>>::new(),
            |mut fan_out, (edge, names)| {
                fan_out.entry(edge.source()).or_default().extend(names.iter());
                fan_out
            },
        );
        let crowded =
            fan_out.into_iter().filter(|(_, names)| !most.allows(names.len())).filter_map(|(source, names)| {
                let site = (*names.values().min()?).clone();
                Some(Violation::new(
                    site,
                    Finding::CrowdedVertex { source: source.clone(), edges: EdgeCount::from(names.len()), most },
                ))
            });
        let structural: Vec<Violation> = duplicates.chain(crowded).collect();
        self.violations.extend(structural);
        self.violations
    }

    fn check_function(&mut self, context: &ImplContext, function: &ImplItemFn) {
        let shape = FunctionShape::describe(&self.path, function, context, self.declarations);
        self.violations.extend(
            matches!(shape.returns(), Returns::TupleHoldingVertex)
                .then(|| Violation::new(shape.site().clone(), Finding::TupleHoldsVertex)),
        );
        let receiver = function.sig.receiver();
        let source = shape.vertex_parameters().first();
        match (receiver, shape.returns().vertex(), source) {
            (None, Some(target), Some(source)) if context.in_trait() => {
                let edge = Edge::new(source.clone(), target.clone());
                self.record_edge(context, function, &shape, edge);
            }
            (None, Some(target), Some(_)) => {
                self.violations
                    .push(Violation::new(shape.site().clone(), Finding::FreeTransition { target: target.clone() }));
            }
            (Some(receiver), _, _) if receiver.reference.is_some() => {
                self.violations.extend(Self::view_fault(context, receiver, &shape));
            }
            (Some(_), Some(target), _) => {
                let edge = context.edge_source(&shape).map(|source| Edge::new(source, target.clone()));
                match edge {
                    None => self.violations.push(Violation::new(
                        shape.site().clone(),
                        Finding::EdgeFromNonVertex { target: target.clone(), owner: context.self_type().clone() },
                    )),
                    Some(edge) => self.record_edge(context, function, &shape, edge),
                }
            }
            _ => {}
        }
    }

    fn record_edge(&mut self, context: &ImplContext, function: &ImplItemFn, shape: &FunctionShape, edge: Edge) {
        if context.is_public(function) {
            self.edges
                .entry(edge)
                .or_default()
                .entry(context.edge_name(function))
                .or_insert_with(|| shape.site().clone());
        } else {
            self.violations.push(Violation::new(shape.site().clone(), Finding::HiddenEdge { edge }));
        }
    }

    fn view_fault(context: &ImplContext, receiver: &Receiver, shape: &FunctionShape) -> Option<Violation> {
        if receiver.mutability.is_some() {
            matches!(context.kind(), ImplKind::InherentOnVertex).then(|| {
                Violation::new(shape.site().clone(), Finding::MutatesVertex { vertex: context.self_type().clone() })
            })
        } else {
            shape.returns().vertex().filter(|_| shape.body() != Body::FieldProjection).map(|target| {
                Violation::new(shape.site().clone(), Finding::ViewReturnsVertex { target: target.clone() })
            })
        }
    }
}

impl<'ast> Visit<'ast> for EdgeScan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.scanned(item));
    }
}

impl EdgeScan<'_> {
    fn scanned(&mut self, item: &ItemImpl) {
        let context = ImplContext::describe(item, self.declarations);
        item.items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(function),
                _ => None,
            })
            .for_each(|function| self.check_function(&context, function));
        syn::visit::visit_item_impl(self, item);
    }
}
