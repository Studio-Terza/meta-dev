use super::quote::Quote;

#[derive(Clone, Copy, Debug)]
pub(in crate::document::command) enum Hidden {
    Nothing,
    Singles,
    Quoted,
}

impl Hidden {
    pub(super) fn keeps(self, before: Quote, after: Quote) -> Option<()> {
        match (self, before, after) {
            (Hidden::Singles, Quote::Single, _) | (Hidden::Quoted, Quote::Single | Quote::Double, _) => None,
            _ => (before == after).then_some(()),
        }
    }
}
