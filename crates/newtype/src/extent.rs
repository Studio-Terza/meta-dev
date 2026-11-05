mod filled;

pub use filled::Filled;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Extent {
    Empty,
    Filled,
}

pub trait Measured {
    fn extent(&self) -> Extent;
}

impl<Item> Measured for [Item] {
    fn extent(&self) -> Extent {
        match self.first() {
            Some(_) => Extent::Filled,
            None => Extent::Empty,
        }
    }
}

impl<Item> Measured for Vec<Item> {
    fn extent(&self) -> Extent {
        self.as_slice().extent()
    }
}

impl Measured for str {
    fn extent(&self) -> Extent {
        self.as_bytes().extent()
    }
}

impl Measured for String {
    fn extent(&self) -> Extent {
        self.as_str().extent()
    }
}

impl<Inner: Measured + ?Sized> Measured for &Inner {
    fn extent(&self) -> Extent {
        (**self).extent()
    }
}
