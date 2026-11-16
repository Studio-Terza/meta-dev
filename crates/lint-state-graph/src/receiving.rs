use syntax::Receiver;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receiving {
    Associated,
    Owned,
    Shared,
    Mutable,
}

impl From<Option<&Receiver>> for Receiving {
    fn from(receiver: Option<&Receiver>) -> Receiving {
        receiver.map_or(Receiving::Associated, Receiving::declared)
    }
}

impl Receiving {
    fn declared(receiver: &Receiver) -> Receiving {
        match receiver.reference {
            None => Receiving::Owned,
            Some(_) => receiver.mutability.map_or(Receiving::Shared, |_| Receiving::Mutable),
        }
    }

    pub fn routed<Routed>(
        self,
        associated: impl FnOnce() -> Routed,
        owned: impl FnOnce() -> Routed,
        viewed: impl FnOnce(Receiving) -> Routed,
    ) -> Routed {
        match self {
            Receiving::Associated => associated(),
            Receiving::Owned => owned(),
            Receiving::Shared | Receiving::Mutable => viewed(self),
        }
    }
}
