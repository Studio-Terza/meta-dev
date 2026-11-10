use transcript_ports::Authorship;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Owner,
    Everyone,
}

impl Scope {
    pub(crate) fn only(self) -> Option<Authorship> {
        match self {
            Scope::Owner => Some(Authorship::Owner),
            Scope::Everyone => None,
        }
    }
}
