use super::Body;

impl Body {
    #[must_use]
    pub fn part(&self, cut: impl for<'text> FnOnce(&'text str) -> &'text str) -> Body {
        Body(cut(&self.0).to_owned())
    }
}
