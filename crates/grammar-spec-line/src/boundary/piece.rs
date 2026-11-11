use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Piece {
    #[token("none")]
    None,
    #[regex("owner [0-9A-Fa-f]{8}[0-9A-Fa-f-]*")]
    Owner,
    #[regex("[ ]*[a-z0-9-]+#[DL][0-9]+(,[ ]*[a-z0-9-]+#[DL][0-9]+)*")]
    Labels,
    #[regex(r"\P{White_Space}+")]
    Word,
    #[regex(r"\p{White_Space}+")]
    Space,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Form {
    None,
    Owner,
    Labels,
}

impl Piece {
    pub(super) fn spoken(self) -> Option<Piece> {
        match self {
            Piece::Space => None,
            Piece::None | Piece::Owner | Piece::Labels | Piece::Word => Some(self),
        }
    }

    pub(super) fn attached(self) -> Option<Form> {
        match self {
            Piece::None => Some(Form::None),
            Piece::Owner => Some(Form::Owner),
            Piece::Labels => Some(Form::Labels),
            Piece::Word | Piece::Space => None,
        }
    }
}
