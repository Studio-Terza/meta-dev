use crate::boundary::Quote;
use crate::resolved::Resolved;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quoted {
    invoked: Resolved,
    quote: Quote,
}

impl Quoted {
    #[must_use]
    pub fn new(invoked: Resolved, quote: Quote) -> Quoted {
        Quoted { invoked, quote }
    }

    #[must_use]
    pub fn invoked(&self) -> &Resolved {
        &self.invoked
    }
}

impl AsRef<Quote> for Quoted {
    fn as_ref(&self) -> &Quote {
        &self.quote
    }
}
