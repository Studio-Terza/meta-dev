use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facet {
    #[name("project")]
    Project,
    #[name("feature")]
    Feature,
    #[name("smell")]
    Smell,
}
