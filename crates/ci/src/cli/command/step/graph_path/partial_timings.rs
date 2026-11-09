use std::fmt;

#[derive(Clone, Copy, Debug)]
pub(super) struct PartialTimings;

impl fmt::Display for PartialTimings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "graph-path skipped: the timings page records a failed build, so it times only part of the clippy units and nothing is measured"
        )
    }
}
