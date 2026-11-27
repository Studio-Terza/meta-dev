mod story;

use proc_macro::TokenStream;
use story::Story;

#[proc_macro_derive(Fixture)]
pub fn derive_fixture(input: TokenStream) -> TokenStream {
    Story::expanded(input.into()).into()
}
