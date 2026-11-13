use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Subject<'model> {
    krate: &'model Name,
    name: Name,
}

impl<'model> Subject<'model> {
    pub(crate) fn new(krate: &'model Name, name: Name) -> Subject<'model> {
        Subject { krate, name }
    }

    pub(crate) fn krate(&self) -> &'model Name {
        self.krate
    }
}

impl AsRef<Name> for Subject<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
