mod named;

use graph::{Mention, PathUse, Placed, Reach, Region};
use lint_core::{Fix, Message, Problem, Violation};
use lint_layout::{PathTail, Word, words};

pub(crate) use named::Named;

#[derive(Debug)]
pub(crate) struct Panics;

impl Panics {
    const UNNAMED: [[Word<'static>; 1]; 10] = [
        words!["panic"],
        words!["unreachable"],
        words!["todo"],
        words!["unimplemented"],
        words!["assert"],
        words!["assert_eq"],
        words!["assert_ne"],
        words!["debug_assert"],
        words!["debug_assert_eq"],
        words!["debug_assert_ne"],
    ];
    const NAMED: [Word<'static>; 1] = words!["invariant"];
    const REFUSED: Message = Message::new(
        Problem::new("a panic that names no invariant"),
        Fix::new("name it: invariant!(holds, Invariant::X), or invariant!(broken Invariant::X) where an arm crashes"),
    );

    pub(crate) fn refused<'model>(placed: Placed<'model, &'model PathUse>) -> Option<Violation<'model>> {
        let used = Self::shipped(placed)?;
        Self::UNNAMED.iter().find_map(|tail| used.spelling(PathTail::new(tail)))?;
        Some(Violation::new(placed.site(), Self::REFUSED))
    }

    fn named(placed: &Placed<'_, &PathUse>) -> Option<()> {
        Self::shipped(*placed)?.spelling(PathTail::new(&Self::NAMED)).map(|_| ())
    }

    fn shipped<'model>(placed: Placed<'model, &'model PathUse>) -> Option<&'model PathUse> {
        let used = placed.entity();
        Some(used.mention()).filter(|mention| *mention == Mention::Invoked)?;
        Some(AsRef::<Region>::as_ref(used).reach()).filter(|reach| *reach == Reach::Shipped).map(|_| used)
    }
}
