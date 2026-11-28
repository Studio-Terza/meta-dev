use timer_ports::Wall;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stamp {
    Born,
    Touched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamps {
    born: Wall,
    touched: Wall,
}

impl Stamps {
    #[must_use]
    pub fn new(born: Wall, touched: Wall) -> Stamps {
        Stamps { born, touched }
    }

    #[must_use]
    pub const fn dated(self, stamp: Stamp) -> Wall {
        match stamp {
            Stamp::Born => self.born,
            Stamp::Touched => self.touched,
        }
    }
}
