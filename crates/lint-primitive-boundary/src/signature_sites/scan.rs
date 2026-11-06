#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
    Scanning,
    Idle,
}

impl Scan {
    pub fn scanning(self) -> Option<Scan> {
        match self {
            Scan::Scanning => Some(self),
            Scan::Idle => None,
        }
    }
}
