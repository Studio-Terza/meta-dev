#![deny(clippy::wildcard_enum_match_arm)]

mod acyclic;
mod annotation;
mod listed;

pub use acyclic::{
    Acyclic, Closed, Closes, Closing, Cycle, Keyed, Refines, Refuses, SegmentCorrects, SegmentSupersedes, Supersedes,
};
pub use annotation::{Annotation, Remarking};
pub use listed::Listed;
