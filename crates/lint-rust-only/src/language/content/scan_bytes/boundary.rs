use file_ports::Bytes;
use newtype::Newtype;

use super::ScanBytes;

impl ScanBytes {
    const BREAK: u8 = b'\n';

    pub(crate) fn first_line(self, contents: &Bytes) -> Option<String> {
        let window = self.bytes(contents);
        let line = window.split_inclusive(|byte| *byte == Self::BREAK).next().unwrap_or_default();
        String::from_utf8(line.to_vec()).ok()
    }
}

impl ScanBytes {
    pub(crate) fn bytes(self, contents: &Bytes) -> Vec<u8> {
        let all = contents.as_ref();
        let limit = usize::try_from(self.into_inner()).unwrap_or(usize::MAX);
        all.get(..limit).unwrap_or(all).to_vec()
    }
}
