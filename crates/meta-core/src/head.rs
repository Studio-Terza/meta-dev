use crate::event::{Event, Record};
use crate::place::Place;

/// What a log's first record says about which log it is. The lead's real log starts with the
/// `RealLog` marker, so its identity travels with its content: a path, a link, HOME or
/// XDG_DATA_HOME cannot hide it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    /// The log is missing or empty.
    Empty,
    /// The first record is the `RealLog` marker.
    Marked,
    /// The first record is any other event.
    Unmarked,
    /// The first record cannot be read or parsed.
    Unreadable,
}

/// What `meta mark-real-log` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marking {
    Added,
    AlreadyMarked,
}

impl Head {
    /// From the log's first record, or why it cannot be read.
    pub fn of<E>(first: Result<Option<&Record>, E>) -> Self {
        match first {
            Err(_) => Self::Unreadable,
            Ok(None) => Self::Empty,
            Ok(Some(Record { event: Event::RealLog, .. })) => Self::Marked,
            Ok(Some(_)) => Self::Unmarked,
        }
    }

    /// Whether only a main build may write the log. A marked log is real wherever it is reached
    /// from. A first record that cannot be read is real: fail closed. The default data dir's log
    /// is real until it is marked, so that the lead's log stays guarded before its one-time marking.
    pub fn is_real(self, place: Place) -> bool {
        match (self, place) {
            (Self::Marked | Self::Unreadable, _) | (_, Place::Default) => true,
            (Self::Empty | Self::Unmarked, Place::Elsewhere) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Head;
    use crate::event::{Event, Record};
    use crate::ids::Timestamp;
    use crate::place::Place;

    #[test]
    fn a_marked_or_unreadable_log_is_real_anywhere_and_an_unmarked_one_only_in_the_default_data_dir() {
        let at = Timestamp::from_unix_seconds(1);
        let marker = Record { at, event: Event::RealLog };
        let other = Record { at, event: Event::Done { task: "task-1".parse().unwrap() } };
        let heads = [
            Head::of::<()>(Ok(None)),
            Head::of::<()>(Ok(Some(&marker))),
            Head::of::<()>(Ok(Some(&other))),
            Head::of(Err(())),
        ];
        assert_eq!(heads, [Head::Empty, Head::Marked, Head::Unmarked, Head::Unreadable]);
        let real = |place| heads.map(|head| head.is_real(place));
        assert_eq!(real(Place::Elsewhere), [false, true, false, true]);
        assert_eq!(real(Place::Default), [true, true, true, true]);
    }

    #[test]
    fn the_marker_is_one_line_with_no_fields_but_its_time() {
        let marker = Record { at: Timestamp::from_unix_seconds(5), event: Event::RealLog };
        assert_eq!(serde_json::to_string(&marker).unwrap(), r#"{"at":5,"event":"RealLog"}"#);
    }
}
