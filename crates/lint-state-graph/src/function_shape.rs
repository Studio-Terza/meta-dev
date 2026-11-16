mod body;
mod returns;
mod written;

pub use body::Body;
pub use returns::{Returns, WrittenReturns};
pub use written::WrittenFn;

use syntax::{ImplItem, ImplItemFn};

use lint_core::Site;
use lint_core::TypeName;

pub struct FunctionShape<'found, 'file> {
    site: Site<'file>,
    returns: Returns<'found>,
    vertex_parameters: Vec<&'found TypeName>,
    body: Body,
}

impl<'found, 'file> FunctionShape<'found, 'file> {
    pub fn function(item: &ImplItem) -> Option<&ImplItemFn> {
        match item {
            ImplItem::Fn(function) => Some(function),
            _ => None,
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
