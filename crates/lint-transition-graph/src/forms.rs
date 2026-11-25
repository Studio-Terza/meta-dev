use crate::edge::Edge;
use crate::form::Role;

#[derive(Debug, Default)]
pub(crate) struct Written<'edge, 'found, 'file>(Vec<&'edge Edge<'found, 'file>>);

#[derive(Debug, Default)]
pub(crate) struct Lifted<'edge, 'found, 'file>(Vec<&'edge Edge<'found, 'file>>);

#[derive(Debug, Default)]
pub(crate) struct Stored<'edge, 'found, 'file>(Vec<&'edge Edge<'found, 'file>>);

impl<'edge, 'found, 'file> FromIterator<&'edge Edge<'found, 'file>> for Written<'edge, 'found, 'file> {
    fn from_iter<Edges: IntoIterator<Item = &'edge Edge<'found, 'file>>>(edges: Edges) -> Self {
        Written(
            edges
                .into_iter()
                .filter(|edge| edge.form().role() != Role::Lift)
                .filter(|edge| edge.form().role() != Role::Store)
                .collect(),
        )
    }
}

impl<'edge, 'found, 'file> FromIterator<&'edge Edge<'found, 'file>> for Lifted<'edge, 'found, 'file> {
    fn from_iter<Edges: IntoIterator<Item = &'edge Edge<'found, 'file>>>(edges: Edges) -> Self {
        Lifted(edges.into_iter().filter(|edge| edge.form().role() == Role::Lift).collect())
    }
}

impl<'edge, 'found, 'file> FromIterator<&'edge Edge<'found, 'file>> for Stored<'edge, 'found, 'file> {
    fn from_iter<Edges: IntoIterator<Item = &'edge Edge<'found, 'file>>>(edges: Edges) -> Self {
        Stored(edges.into_iter().filter(|edge| edge.form().role() == Role::Store).collect())
    }
}

impl<'edge, 'found, 'file> IntoIterator for Written<'edge, 'found, 'file> {
    type Item = &'edge Edge<'found, 'file>;
    type IntoIter = std::vec::IntoIter<&'edge Edge<'found, 'file>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'edge, 'found, 'file> IntoIterator for Lifted<'edge, 'found, 'file> {
    type Item = &'edge Edge<'found, 'file>;
    type IntoIter = std::vec::IntoIter<&'edge Edge<'found, 'file>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'edge, 'found, 'file> IntoIterator for Stored<'edge, 'found, 'file> {
    type Item = &'edge Edge<'found, 'file>;
    type IntoIter = std::vec::IntoIter<&'edge Edge<'found, 'file>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
