pub trait Semigroup: Sized {
    #[must_use]
    fn combine(self, other: Self) -> Self;
}

pub trait Monoid: Semigroup + Default {
    fn concat(parts: impl IntoIterator<Item = Self>) -> Self {
        parts.into_iter().fold(Self::default(), Self::combine)
    }
}

impl Semigroup for () {
    fn combine(self, (): ()) {}
}

impl Monoid for () {}

impl<Item> Semigroup for Vec<Item> {
    fn combine(mut self, other: Vec<Item>) -> Vec<Item> {
        self.extend(other);
        self
    }
}

impl<Item> Monoid for Vec<Item> {}

#[derive(Debug, Default)]
pub enum Catenation<Item> {
    #[default]
    Empty,
    Leaf(Vec<Item>),
    Joined(Box<Catenation<Item>>, Box<Catenation<Item>>),
}

#[derive(Debug)]
pub struct Leaves<Item> {
    pending: Vec<Catenation<Item>>,
}

#[derive(Debug)]
struct Opened<Item> {
    leaf: Vec<Item>,
    children: Vec<Catenation<Item>>,
}

impl<Item> Semigroup for Catenation<Item> {
    fn combine(self, other: Catenation<Item>) -> Catenation<Item> {
        match self.occupied() {
            Some(first) => Catenation::after(first, other.occupied()),
            None => other,
        }
    }
}

impl<Item> Monoid for Catenation<Item> {}

impl<Item> From<Vec<Item>> for Catenation<Item> {
    fn from(items: Vec<Item>) -> Catenation<Item> {
        Catenation::Leaf(items)
    }
}

impl<Item> FromIterator<Item> for Catenation<Item> {
    fn from_iter<Items: IntoIterator<Item = Item>>(items: Items) -> Catenation<Item> {
        Catenation::Leaf(items.into_iter().collect())
    }
}

impl<Item> IntoIterator for Catenation<Item> {
    type Item = Item;
    type IntoIter = std::iter::Flatten<Leaves<Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Leaves { pending: vec![self] }.flatten()
    }
}

impl<Item> Catenation<Item> {
    fn opened(self) -> Opened<Item> {
        match self {
            Catenation::Empty => Opened { leaf: Vec::new(), children: Vec::new() },
            Catenation::Leaf(leaf) => Opened { leaf, children: Vec::new() },
            Catenation::Joined(first, second) => Opened { leaf: Vec::new(), children: vec![*second, *first] },
        }
    }
}

impl<Item> Iterator for Leaves<Item> {
    type Item = Vec<Item>;

    fn next(&mut self) -> Option<Vec<Item>> {
        let Opened { leaf, children } = self.pending.pop()?.opened();
        self.pending.extend(children);
        Some(leaf)
    }
}

impl<Item> Catenation<Item> {
    fn occupied(self) -> Option<Catenation<Item>> {
        match self {
            Catenation::Empty => None,
            Catenation::Leaf(leaf) => leaf.first().map(|_| ()).map(|()| Catenation::Leaf(leaf)),
            joined @ Catenation::Joined(..) => Some(joined),
        }
    }
}

impl<Item> Catenation<Item> {
    fn after(first: Catenation<Item>, second: Option<Catenation<Item>>) -> Catenation<Item> {
        match second {
            Some(tail) => Catenation::Joined(Box::new(first), Box::new(tail)),
            None => first,
        }
    }
}
