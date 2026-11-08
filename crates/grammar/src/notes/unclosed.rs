use newtype::{Arbitrary, Newtype};

use super::Quote;

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the quote opened before {0} never closes")]
pub struct Unclosed(Quote);
