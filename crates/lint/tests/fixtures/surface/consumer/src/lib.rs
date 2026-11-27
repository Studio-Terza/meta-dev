use provider::{Imported, Renamed as Local};
use globbed::*;
pub struct Unseen;
macro_rules! made {
    ($name:ident) => {
        let $name = provider::Transcribed;
    };
    ($($item:expr),*) => {
        $( $item; )*
    };
}
