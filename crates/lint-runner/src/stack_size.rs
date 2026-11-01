#[derive(Clone, Copy, Debug)]
pub struct StackSize(usize);

impl StackSize {
    const BYTES_PER_MEBIBYTE: usize = 1024 * 1024;
    const LINT_MEBIBYTES: usize = 256;

    pub const LINT: StackSize = StackSize(Self::LINT_MEBIBYTES * Self::BYTES_PER_MEBIBYTE);

    #[must_use]
    pub const fn bytes(self) -> usize {
        self.0
    }
}
