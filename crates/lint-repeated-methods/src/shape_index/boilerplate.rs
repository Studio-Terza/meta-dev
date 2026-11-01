use syn::Path;

#[derive(Debug)]
pub struct Boilerplate;

impl Boilerplate {
    const TRAITS: [&'static str; 20] = [
        "Add",
        "Sub",
        "Mul",
        "Div",
        "Rem",
        "Neg",
        "Not",
        "AddAssign",
        "SubAssign",
        "MulAssign",
        "DivAssign",
        "RemAssign",
        "Sum",
        "Product",
        "Display",
        "Default",
        "From",
        "AsRef",
        "Deref",
        "FromStr",
    ];

    pub fn names(path: &Path) -> bool {
        path.segments.last().is_some_and(|last| Self::TRAITS.iter().any(|name| last.ident == name))
    }
}
