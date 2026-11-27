mod boundary;
mod cases;
mod laws;
mod properties;
mod rows;

use serde::de::DeserializeOwned;

pub use boundary::{CaseName, Debugged, Drift, Parsed, Spelled, Story};
pub use cases::{Case, Cases};
pub use fixture_derive::Fixture;
pub use laws::Laws;
pub use properties::Properties;
pub use rows::Rows;
pub use test_failure::Failed;

pub trait Fixture: DeserializeOwned {
    const STORY: Story;
}
