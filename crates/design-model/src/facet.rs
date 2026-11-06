use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facet {
    #[name("project")]
    Project,
    #[name("feature")]
    Feature,
    #[name("smell")]
    Smell,
}
