use super::Quote;

impl Quote {
    pub(in crate::document::command::unquoted) fn after(self, letter: char) -> Quote {
        match (self, letter) {
            (Quote::Outside, '\'') => Quote::Single,
            (Quote::Outside, '"') => Quote::Double,
            (Quote::Single, '\'') | (Quote::Double, '"') => Quote::Outside,
            _ => self,
        }
    }
}
