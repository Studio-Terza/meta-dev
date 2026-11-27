#[proc_macro_derive(Shown)]
pub fn derive_shown(input: TokenStream) -> TokenStream {
    let called = format_ident!("spelled_late");
    let skipped = format_ident!("input{}", 1);
    quote!(::library::#called())
}
