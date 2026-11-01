use std::fmt;

use body_shape::TokenCount;

pub(super) struct Finding<'sites> {
    size: TokenCount,
    others: &'sites [String],
}

impl<'sites> Finding<'sites> {
    pub(super) fn new(size: TokenCount, others: &'sites [String]) -> Finding<'sites> {
        Finding { size, others }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} tokens in {} copies, also at {}; fix with a standard trait such as From, TryFrom, FromStr or Display, else a generic over an existing trait, else a derive",
            self.size,
            self.others.len() + 1,
            self.others.join(", ")
        )
    }
}
