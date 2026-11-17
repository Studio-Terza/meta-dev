#[derive(Debug)]
pub struct Drawn<Kept, Out> {
    kept: Kept,
    out: Out,
}

impl<Kept, Out> Drawn<Kept, Out> {
    pub fn new(kept: Kept, out: Out) -> Drawn<Kept, Out> {
        Drawn { kept, out }
    }

    pub(crate) fn parted(self, keep: impl FnOnce(Kept)) -> Out {
        keep(self.kept);
        self.out
    }
}
