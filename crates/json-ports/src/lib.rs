mod boundary;
mod codec;
mod json;

pub use boundary::{Document, Fields, Lenient, Object, Opaque};
pub use codec::{Decode, Encode};
pub use json::{Json, JsonFailure};
