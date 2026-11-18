use std::collections::BTreeSet;
use std::vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event<Index> {
    Rooted(Index),
    Entered(Index),
    Finished(Index),
}

#[derive(Debug)]
pub struct Trees<Index>(Vec<Vec<Index>>);

#[derive(Debug)]
struct Frame<Index, Pending> {
    node: Index,
    pending: Pending,
}

#[derive(Debug)]
struct Walk<Index, Roots, Neighbours, Pending> {
    neighbours: Neighbours,
    roots: Roots,
    seen: BTreeSet<Index>,
    path: Vec<Frame<Index, Pending>>,
}

#[derive(Clone, Copy, Debug)]
struct Opened<Index> {
    node: Index,
    rooted: Option<()>,
}

impl<Index: Copy + Ord> Event<Index> {
    pub fn walked<Roots, Neighbours, Pending>(
        roots: Roots,
        neighbours: Neighbours,
    ) -> impl Iterator<Item = Event<Index>>
    where
        Roots: IntoIterator<Item = Index>,
        Neighbours: Fn(Index) -> Pending,
        Pending: Iterator<Item = Index>,
    {
        Walk { neighbours, roots: roots.into_iter(), seen: BTreeSet::new(), path: Vec::new() }
    }
}

impl<Index> Event<Index> {
    pub fn finished(self) -> Option<Index> {
        match self {
            Event::Finished(node) => Some(node),
            Event::Rooted(_) | Event::Entered(_) => None,
        }
    }

    fn opened(self) -> Option<Opened<Index>> {
        match self {
            Event::Rooted(node) => Some(Opened { node, rooted: Some(()) }),
            Event::Entered(node) => Some(Opened { node, rooted: None }),
            Event::Finished(_) => None,
        }
    }
}

impl<Index: Copy> FromIterator<Event<Index>> for Trees<Index> {
    fn from_iter<Events: IntoIterator<Item = Event<Index>>>(events: Events) -> Trees<Index> {
        let grouped = events.into_iter().fold(Vec::new(), |mut trees: Vec<Vec<Index>>, event| {
            trees.extend(event.opened().and_then(|opened| opened.rooted).map(|()| Vec::new()));
            trees.last_mut().into_iter().zip(event.finished()).for_each(|(tree, node)| tree.push(node));
            trees
        });
        Trees(grouped)
    }
}

impl<Index> IntoIterator for Trees<Index> {
    type Item = Vec<Index>;
    type IntoIter = vec::IntoIter<Vec<Index>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Index, Roots, Neighbours, Pending> Iterator for Walk<Index, Roots, Neighbours, Pending>
where
    Index: Copy + Ord,
    Roots: Iterator<Item = Index>,
    Neighbours: Fn(Index) -> Pending,
    Pending: Iterator<Item = Index>,
{
    type Item = Event<Index>;

    fn next(&mut self) -> Option<Event<Index>> {
        let Walk { neighbours, roots, seen, path } = self;
        let below = path.last_mut().map(|frame| frame.pending.find(|child| seen.insert(*child)));
        let entered = below.flatten().map(Event::Entered);
        let finished = below.filter(Option::is_none).and_then(|_| path.pop()).map(|done| Event::Finished(done.node));
        let rooted = below.map_or_else(|| roots.find(|root| seen.insert(*root)).map(Event::Rooted), |_| None);
        let event = entered.or(finished).or(rooted);
        let opened =
            event.and_then(Event::opened).map(|opened| Frame { node: opened.node, pending: neighbours(opened.node) });
        path.extend(opened);
        event
    }
}
