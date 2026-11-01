mod shape_token;

use quote::ToTokens;
use syn::ItemFn;

use shape_token::ShapeToken;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BodyShape(Vec<ShapeToken>);

impl From<&ItemFn> for BodyShape {
    fn from(function: &ItemFn) -> BodyShape {
        let attributes = function.attrs.iter().map(ToTokens::to_token_stream);
        BodyShape(attributes.chain([function.block.to_token_stream()]).flat_map(ShapeToken::normalized).collect())
    }
}
