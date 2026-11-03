use newtype::Newtype;

use crate::word::Word;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathTail<'words>(&'words [Word<'words>]);
