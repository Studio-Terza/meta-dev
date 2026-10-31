use std::fmt;

pub enum Finding {
    Logic,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::Logic = self;
        formatter.write_str("main.rs holds only fn main and imports; move this into the library")
    }
}
