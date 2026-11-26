use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[strum(serialize_all = "kebab-case")]
pub enum Relation {
    RepliesTo,
    Answers,
    Cites,
    Resends,
}
