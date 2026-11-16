#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;

pub use boundary::{Classes, Lexed, Line, Location, Piece, Pieces, Quote};
