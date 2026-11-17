use invariants::invariant;

pub struct Parts;

impl Parts {
    pub fn checked(self) -> Parts {
        invariant!(broken Invariant::Counted);
    }
}
