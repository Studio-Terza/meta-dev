use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Facet {
    Project,
    Feature,
    Smell,
}
