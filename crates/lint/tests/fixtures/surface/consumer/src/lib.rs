use provider::{Imported, Renamed as Local};
use globbed::*;
pub struct Unseen;
macro_rules! made {
    ($name:ident) => {
        let $name = 1;
    };
    ($($item:expr),*) => {
        $( provider::Transcribed::from($item) ),*
    };
}
