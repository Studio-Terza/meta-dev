use super::ColumnName;

impl TryFrom<String> for ColumnName {
    type Error = syn::Error;

    fn try_from(name: String) -> syn::Result<ColumnName> {
        syn::parse_str(&name)
    }
}
