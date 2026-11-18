use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facet {
    Project,
    Feature,
    Smell,
}
