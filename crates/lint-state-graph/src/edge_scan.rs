use std::collections::BTreeMap;
use std::path::Path;

use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, Receiver};

use super::declarations::Declarations;
use super::edge::Edge;
use super::edge_count::EdgeCount;
use super::finding::Finding;
use super::function_shape::{Body, FunctionShape, Returns};
use super::impl_context::{ImplContext, ImplKind};
use lint_core::SourceFile;
use lint_core::TypeName;
use lint_core::Violation;
use lint_core::Walk;
use lint_core::{Bound, Site};
use lint_layout::{Length, Name};

pub struct EdgeScan<'scan, 'file> {
    declarations: &'scan Declarations<'scan>,
    path: &'file Path,
    edges: BTreeMap<Edge<'scan>, BTreeMap<Name, Site<'file>>>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> EdgeScan<'scan, 'file> {
    const EDGES_PER_VERTEX: Bound = Bound::Most(4);

    pub fn new(declarations: &'scan Declarations<'scan>) -> Self {
        EdgeScan { declarations, path: Path::new(""), edges: BTreeMap::new(), violations: Vec::new() }
    }

    pub fn scan(&mut self, file: &'file SourceFile) {
        self.path = file.path();
        self.visit_file(file.syntax());
    }

    pub fn violations(mut self) -> Vec<Violation<'file>> {
        let most = Self::EDGES_PER_VERTEX;
        let duplicates = self.edges.iter().filter(|(_, names)| names.len() > 1).filter_map(|(edge, names)| {
            let site = *names.values().min()?;
            Some(Violation::new(site, Finding::DuplicateEdge { edge, sites: names }))
        });
        let fan_out = self.edges.iter().fold(
            BTreeMap::<&TypeName, BTreeMap<&Name, &Site<'file>>>::new(),
            |mut fan_out, (edge, names)| {
                fan_out.entry(edge.source()).or_default().extend(names.iter());
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
        self.violations.extend(structural);
        self.violations
    }

    fn check_function(&mut self, context: &ImplContext<'scan>, function: &ImplItemFn) {
        let shape = FunctionShape::describe(self.path, function, context, self.declarations);
        self.violations.extend(
            matches!(shape.returns(), Returns::TupleHoldingVertex)
                .then(|| Violation::new(shape.site(), Finding::TupleHoldsVertex)),
        );
        let receiver = function.sig.receiver();
        let source = shape.vertex_parameters().first();
        match (receiver, shape.returns().vertex(), source) {
            (None, Some(target), Some(source)) if context.kind().trait_name().is_some() => {
                let edge = Edge::new(source, target);
                self.record_edge(context, function, &shape, edge);
            }
            (None, Some(target), Some(_)) => {
                self.violations.push(Violation::new(shape.site(), Finding::FreeTransition { target }));
            }
            (Some(receiver), _, _) if receiver.reference.is_some() => {
                self.violations.extend(Self::view_fault(context, receiver, &shape));
            }
            (Some(_), Some(target), _) => {
                let edge = context.edge_source(&shape).map(|source| Edge::new(source, target));
                match edge {
                    None => self.violations.push(Violation::new(
                        shape.site(),
                        Finding::EdgeFromNonVertex { target, owner: context.self_type() },
                    )),
                    Some(edge) => self.record_edge(context, function, &shape, edge),
                }
            }
            _ => {}
        }
    }

    fn record_edge(
        &mut self,
        context: &ImplContext<'scan>,
        function: &ImplItemFn,
        shape: &FunctionShape<'scan, 'file>,
        edge: Edge<'scan>,
    ) {
        match context.public(function) {
            Some(function) => {
                self.edges.entry(edge).or_default().entry(context.edge_name(function)).or_insert_with(|| shape.site());
            }
            None => self.violations.push(Violation::new(shape.site(), Finding::HiddenEdge { edge })),
        }
    }

    fn view_fault(
        context: &ImplContext<'scan>,
        receiver: &Receiver,
        shape: &FunctionShape<'scan, 'file>,
    ) -> Option<Violation<'file>> {
        if receiver.mutability.is_some() {
            matches!(context.kind(), ImplKind::InherentOnVertex(_))
                .then(|| Violation::new(shape.site(), Finding::MutatesVertex { vertex: context.self_type() }))
        } else {
            shape
                .returns()
                .vertex()
                .filter(|_| shape.body() != Body::FieldProjection)
                .map(|target| Violation::new(shape.site(), Finding::ViewReturnsVertex { target }))
        }
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
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(function),
                _ => None,
            })
            .for_each(|function| self.check_function(&context, function));
        syn::visit::visit_item_impl(self, item);
    }
}
