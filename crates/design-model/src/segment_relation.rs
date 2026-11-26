use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub enum SegmentRelation {
    Answers,
    RefersTo,
    Supersedes,
    Corrects,
}
