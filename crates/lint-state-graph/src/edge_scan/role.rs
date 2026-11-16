use super::EdgeResolution;
use crate::edge::Edge;
use crate::finding::Finding;
use crate::function_shape::FunctionShape;
use crate::function_shape::WrittenFn;
use crate::impl_context::ImplContext;
use crate::receiving::Receiving;
use lint_core::{TypeName, Violation};

#[derive(Clone, Copy, Debug)]
pub enum Role<'scan> {
    Edge(Edge<'scan>),
    Free(&'scan TypeName),
    View(Receiving),
    Unsourced(&'scan TypeName),
    Silent,
}

impl<'scan> Role<'scan> {
    pub fn classified(
        receiver: Receiving,
        context: &ImplContext<'scan>,
        shape: &FunctionShape<'scan, '_>,
    ) -> Role<'scan> {
        receiver.routed(
            || Self::associated(context, shape),
            || shape.returns().vertex().map_or(Role::Silent, |target| Self::consumed(target, context, shape)),
            Role::View,
        )
    }

    pub fn checked<'file>(
        self,
        scan: &mut EdgeResolution<'scan, 'file>,
        context: &ImplContext<'scan>,
        function: &WrittenFn,
        shape: &FunctionShape<'scan, 'file>,
    ) {
        match self {
            Role::Edge(edge) => scan.record_edge(context, function, shape, edge),
            Role::Free(target) => {
                scan.violations.push(Violation::new(shape.site(), Finding::FreeTransition { target }));
            }
            Role::View(receiver) => scan.violations.extend(EdgeResolution::view_fault(context, receiver, shape)),
            Role::Unsourced(target) => scan
                .violations
                .push(Violation::new(shape.site(), Finding::EdgeFromNonVertex { target, owner: context.self_type() })),
            Role::Silent => {}
        }
    }

    fn associated(context: &ImplContext<'scan>, shape: &FunctionShape<'scan, '_>) -> Role<'scan> {
        shape.returns().vertex().zip(shape.vertex_parameters().first()).map_or(Role::Silent, |(target, source)| {
            context.kind().trait_name().map_or(Role::Free(target), |_| Role::Edge(Edge::new(source, target)))
        })
    }

    fn consumed(
        target: &'scan TypeName,
        context: &ImplContext<'scan>,
        shape: &FunctionShape<'scan, '_>,
    ) -> Role<'scan> {
        context.edge_source(shape).map_or(Role::Unsourced(target), |source| Role::Edge(Edge::new(source, target)))
    }
}
