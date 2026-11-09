use logos::Logos;

use super::{Attachment, Opening};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Piece {
    #[token("repair: main")]
    Repair,
    #[token("spec: ")]
    Spec,
    #[token("none")]
    None,
    #[regex("owner [0-9A-Fa-f]{8}[0-9A-Fa-f-]*")]
    Owner,
    #[regex("[ ]*[a-z0-9-]+#[DL][0-9]+(,[ ]*[a-z0-9-]+#[DL][0-9]+)*")]
    Labels,
    #[regex(r"\p{White_Space}+")]
    Space,
}

impl Piece {
    pub(super) fn opening(self) -> Opening {
        match self {
            Piece::Repair => Opening::Repair,
            Piece::Spec => Opening::Spec,
            Piece::None | Piece::Owner | Piece::Labels | Piece::Space => Opening::Other,
        }
    }

    pub(super) fn spoken(self) -> Option<Piece> {
        match self {
            Piece::Space => None,
            Piece::Repair | Piece::Spec | Piece::None | Piece::Owner | Piece::Labels => Some(self),
        }
    }

    pub(super) fn attachment(self, value: &str) -> Option<Attachment<'_>> {
        match self {
            Piece::None | Piece::Owner | Piece::Labels => Some(Attachment(value)),
            Piece::Repair | Piece::Spec | Piece::Space => None,
        }
    }
}
