mod body;
mod returns;

pub use body::Body;
pub use returns::Returns;

use crate::declarations::Vertex;
use std::path::Path;

use syn::{FnArg, ImplItemFn, Type};

use super::declarations::Declarations;
use super::impl_context::ImplContext;
use lint_core::Site;
use lint_core::TypeName;
use lint_layout::LineNumber;

pub struct FunctionShape<'found, 'file> {
    site: Site<'file>,
    returns: Returns<'found>,
    vertex_parameters: Vec<&'found TypeName>,
    body: Body,
}

impl<'found, 'file> FunctionShape<'found, 'file> {
    pub fn describe(
        path: &'file Path,
        function: &ImplItemFn,
        context: &ImplContext<'_>,
        declarations: &'found Declarations,
    ) -> FunctionShape<'found, 'file> {
        let vertex = |ty: &Type| {
            declarations
                .vertex_named(&declarations.resolve(TypeName::from_type(ty).or_self(context.self_type())))
                .map(Vertex::name)
        };
        FunctionShape {
            site: Site::new(path, LineNumber::from(function.sig.ident.span())),
            returns: Returns::classify(&function.sig.output, vertex),
            vertex_parameters: function
                .sig
                .inputs
                .iter()
                .filter_map(|argument| match argument {
                    FnArg::Typed(typed) => vertex(&typed.ty),
                    FnArg::Receiver(_) => None,
                })
                .collect(),
            body: Body::classify(&function.block),
        }
    }

    pub fn site(&self) -> Site<'file> {
        self.site
    }

    pub fn returns(&self) -> &Returns<'found> {
        &self.returns
    }

    pub fn vertex_parameters(&self) -> &[&'found TypeName] {
        &self.vertex_parameters
    }

    pub fn body(&self) -> Body {
        self.body
    }
}
