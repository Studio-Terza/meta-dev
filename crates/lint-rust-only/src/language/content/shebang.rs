mod boundary;

use file_ports::Bytes;

use super::scan_bytes::ScanBytes;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Shebang(Option<String>);

impl Shebang {
    const MARK: &'static str = "#!";
    const ATTRIBUTE: &'static str = "#![";
    const SCANNED: ScanBytes = ScanBytes::new(256);
    const RUST: [&'static str; 2] = ["rust-script", "cargo"];

    pub(crate) fn read_from(contents: &Bytes) -> Option<Self> {
        let line = Self::SCANNED.first_line(contents)?;
        line.get(..Self::MARK.len())
            .filter(|start| *start == Self::MARK)
            .filter(|_| line.get(..Self::ATTRIBUTE.len()) != Some(Self::ATTRIBUTE))
            .and(line.get(Self::MARK.len()..))?
            .parse()
            .ok()
    }

    pub(crate) fn foreign(self) -> Option<Self> {
        match self.0.as_deref().filter(|name| Self::RUST.contains(name)) {
            Some(_) => None,
            None => Some(self),
        }
    }
}
