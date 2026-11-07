use syn::{ImplItemFn, Receiver};

use super::EdgeScan;
use crate::edge::Edge;
use crate::finding::Finding;
use crate::function_shape::FunctionShape;
use crate::impl_context::ImplContext;
use lint_core::{TypeName, Violation};

#[derive(Clone, Copy, Debug)]
pub enum Role<'scan, 'item> {
    Edge(Edge<'scan>),
    Free(&'scan TypeName),
    View(&'item Receiver),
    Unsourced(&'scan TypeName),
    Silent,
}

impl<'scan, 'item> Role<'scan, 'item> {
    pub fn classified(
        receiver: Option<&'item Receiver>,
        context: &ImplContext<'scan>,
        shape: &FunctionShape<'scan, '_>,
    ) -> Self {
        match receiver {
            None => Self::associated(context, shape),
            Some(receiver) => Self::received(receiver, context, shape),
        }
    }

    pub fn checked<'file>(
        self,
        scan: &mut EdgeScan<'scan, 'file>,
        context: &ImplContext<'scan>,
        function: &ImplItemFn,
        shape: &FunctionShape<'scan, 'file>,
    ) {
        match self {
            Role::Edge(edge) => scan.record_edge(context, function, shape, edge),
            Role::Free(target) => {
                scan.violations.push(Violation::new(shape.site(), Finding::FreeTransition { target }));
            }
            Role::View(receiver) => scan.violations.extend(EdgeScan::view_fault(context, receiver, shape)),
            Role::Unsourced(target) => scan
                .violations
                .push(Violation::new(shape.site(), Finding::EdgeFromNonVertex { target, owner: context.self_type() })),
            Role::Silent => {}
        }
    }

    fn associated(context: &ImplContext<'scan>, shape: &FunctionShape<'scan, '_>) -> Self {
        shape.returns().vertex().zip(shape.vertex_parameters().first()).map_or(Role::Silent, |(target, source)| {
            context.kind().trait_name().map_or(Role::Free(target), |_| Role::Edge(Edge::new(source, target)))
        })
    }

    fn received(receiver: &'item Receiver, context: &ImplContext<'scan>, shape: &FunctionShape<'scan, '_>) -> Self {
        receiver.reference.as_ref().map_or_else(
            || shape.returns().vertex().map_or(Role::Silent, |target| Self::consumed(target, context, shape)),
            |_| Role::View(receiver),
        )
    }

    fn consumed(target: &'scan TypeName, context: &ImplContext<'scan>, shape: &FunctionShape<'scan, '_>) -> Self {
        context.edge_source(shape).map_or(Role::Unsourced(target), |source| Role::Edge(Edge::new(source, target)))
    }
}
