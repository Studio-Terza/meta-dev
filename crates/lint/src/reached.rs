use syn::visit::{self, Visit};
use syn::{TypePath, TypeTuple};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Reach<'ast> {
    Path(&'ast TypePath),
    Tuple(&'ast TypeTuple),
}

#[derive(Debug, Default)]
pub(crate) struct Reached<'ast>(Vec<Reach<'ast>>);

impl<'ast> Reached<'ast> {
    pub(crate) fn within(walk: impl FnOnce(&mut Reached<'ast>)) -> Reached<'ast> {
        let mut reached = Reached::default();
        walk(&mut reached);
        reached
    }

    pub(crate) fn reaches(self) -> impl Iterator<Item = Reach<'ast>> {
        self.0.into_iter()
    }

    pub(crate) fn paths(self) -> impl Iterator<Item = &'ast TypePath> {
        self.reaches().filter_map(|reach| match reach {
            Reach::Path(path) => Some(path),
            Reach::Tuple(_) => None,
        })
    }
}

impl<'ast> Visit<'ast> for Reached<'ast> {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.push(Reach::Path(path));
        visit::visit_type_path(self, path);
    }

    fn visit_type_tuple(&mut self, tuple: &'ast TypeTuple) {
        self.0.push(Reach::Tuple(tuple));
        visit::visit_type_tuple(self, tuple);
    }
}
