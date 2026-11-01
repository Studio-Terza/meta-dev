#[proc_macro_derive(Foo)]
pub fn foo(input: TokenStream) -> TokenStream { input }
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn t() {}
}
