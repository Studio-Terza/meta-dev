mod boundary;
mod cases;
mod rows;

use serde::de::DeserializeOwned;

pub use boundary::{CaseName, Drift, Golden, Story};
pub use cases::{Case, Cases};
pub use fixture_derive::Fixture;
pub use rows::Rows;
pub use test_failure::Failed;

pub trait Fixture: DeserializeOwned {
    const STORY: Story;
}
