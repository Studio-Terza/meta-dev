#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Within,
    Warned,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit<Measured: PartialOrd> {
    warn: Measured,
    block: Measured,
}

impl<Measured: PartialOrd + Copy> Limit<Measured> {
    pub const fn at(most: Measured) -> Limit<Measured> {
        Limit { warn: most, block: most }
    }

    pub const fn warned(warn: Measured, block: Measured) -> Limit<Measured> {
        Limit { warn, block }
    }

    pub fn judged(&self, measured: &Measured) -> Verdict {
        if *measured > self.block {
            Verdict::Over
        } else if *measured > self.warn {
            Verdict::Warned
        } else {
            Verdict::Within
        }
    }
}
