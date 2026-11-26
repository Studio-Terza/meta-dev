mod boundary;

use design_model::FeatureName;
use port_core::Op;

pub use boundary::Feature;

#[derive(Clone, Copy, Debug, Op)]
pub struct Features;

#[derive(Debug, Op)]
pub struct Featured(Vec<Feature>);

#[derive(Clone, Debug, Op)]
pub struct FeatureNamed(FeatureName);

impl From<Vec<Feature>> for Featured {
    fn from(features: Vec<Feature>) -> Featured {
        Featured(features)
    }
}

impl From<FeatureName> for FeatureNamed {
    fn from(name: FeatureName) -> FeatureNamed {
        FeatureNamed(name)
    }
}
