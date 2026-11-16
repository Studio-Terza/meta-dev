use std::path::Path;

use syntax::{FnArg, ImplItemFn, Visibility};

use super::{Body, FunctionShape, Returns, WrittenReturns};
use crate::declarations::{Declarations, Vertex};
use crate::impl_context::ImplContext;
use crate::receiving::Receiving;
use lint_core::{Site, TypeName};
use lint_layout::{LineNumber, Name};

#[derive(Debug)]
pub struct WrittenFn {
    line: LineNumber,
    name: Name,
    public: Option<()>,
    receiver: Receiving,
    returns: WrittenReturns,
    parameters: Vec<TypeName>,
    body: Body,
}

impl From<&ImplItemFn> for WrittenFn {
    fn from(function: &ImplItemFn) -> WrittenFn {
        WrittenFn {
            line: LineNumber::from(function.sig.ident.span()),
            name: Name::from(&function.sig.ident),
            public: matches!(function.vis, Visibility::Public(_)).then_some(()),
            receiver: Receiving::from(function.sig.receiver()),
            returns: WrittenReturns::from(&function.sig.output),
            parameters: function.sig.inputs.iter().filter_map(WrittenFn::typed).collect(),
            body: Body::classify(&function.block),
        }
    }
}

impl WrittenFn {
    fn typed(argument: &FnArg) -> Option<TypeName> {
        match argument {
            FnArg::Typed(typed) => Some(TypeName::from_type(&typed.ty)),
            FnArg::Receiver(_) => None,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn public(&self) -> Option<()> {
        self.public
    }

    pub fn receiver(&self) -> Receiving {
        self.receiver
    }
}

impl<'found, 'file> FunctionShape<'found, 'file> {
    pub fn resolved(
        path: &'file Path,
        function: &WrittenFn,
        context: &ImplContext<'_>,
        declarations: &'found Declarations,
    ) -> FunctionShape<'found, 'file> {
        let vertex = |written: &TypeName| {
            declarations.vertex_named(declarations.resolved(written.or_self(context.self_type()))).map(Vertex::name)
        };
        FunctionShape {
            site: Site::new(path, function.line),
            returns: Returns::resolved(&function.returns, vertex),
            vertex_parameters: function.parameters.iter().filter_map(vertex).collect(),
            body: function.body,
        }
    }
}
