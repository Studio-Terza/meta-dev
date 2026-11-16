use crate::source_file::SourceFile;

pub trait Sourced<'file> {
    fn file(self) -> &'file SourceFile;
}
