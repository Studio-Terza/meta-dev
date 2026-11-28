mod lexeme;
mod occurrence;
mod reading;

use occurrence::{Occurrence, Walk};
use reading::Reading;

pub use occurrence::{Construct, Directory, Edit, Edited, Editor, Flagged, Nested, Unreadable};

impl Nested {
    #[must_use]
    pub fn read(command: &str) -> Option<Nested> {
        Reading::occurrences(command).iter().find_map(Occurrence::nested)
    }
}

impl Unreadable {
    #[must_use]
    pub fn read(command: &str) -> Option<Unreadable> {
        Reading::occurrences(command).iter().find_map(Occurrence::unreadable)
    }
}

impl Edited {
    #[must_use]
    pub fn read(command: &str) -> Vec<Edited> {
        Reading::occurrences(command).into_iter().fold(Walk::default(), Walk::stepped).edited()
    }
}
