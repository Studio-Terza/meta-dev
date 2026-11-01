use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub(crate) struct ScanBytes(u64);

impl ScanBytes {
    const BREAK: u8 = b'\n';

    pub(crate) fn first_line(self, path: &Path) -> Option<String> {
        let mut line = Vec::new();
        BufReader::new(File::open(path).ok()?.take(self.into_inner())).read_until(Self::BREAK, &mut line).ok()?;
        String::from_utf8(line).ok()
    }
}

impl ScanBytes {
    pub(crate) fn bytes(self, path: &Path) -> Option<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(path).ok()?.take(self.into_inner()).read_to_end(&mut bytes).ok()?;
        Some(bytes)
    }
}
