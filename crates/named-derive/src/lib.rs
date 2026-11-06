mod table;

use proc_macro::TokenStream;
use table::Table;

#[proc_macro_derive(Named, attributes(name))]
pub fn derive_named(input: TokenStream) -> TokenStream {
    Table::expanded(input.into()).into()
}
