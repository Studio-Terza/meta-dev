macro_rules! cases {
    () => {
        #[test]
        fn generated() {}
    };
}
cases!();
pub struct Shown {
    #[cfg(test)]
    pub probe: u8,
    #[doc(hidden)]
    pub(crate) hidden: u8,
    kept: u8,
}
#[cfg(test)]
impl Shown {
    fn helper() {}
}
#[cfg(test)]
impl Shown {
    pub fn gated_member() {}
    #[doc(hidden)]
    pub fn hidden_member() {}
}
impl Shown {
    #[cfg(test)]
    pub fn gated() {}
    #[doc = "shown"]
    pub fn real() {}
    pub const LIMIT: u8 = 1;
    pub fn wrapped() {
        quick! {
            #[test]
            fn listed() {}
        }
        hidden!();
        crate::cases!();
    }
}
#[cfg(test)]
fn gated_free() {}
#[cfg(test)]
pub mod tests {
    pub fn inside() {}
    #[doc(hidden)]
    pub fn hidden_inside() {}
    macro_rules! hidden {
        () => {
            #[rstest]
            fn emitted() {}
        };
    }
}
#[cfg_attr(feature = "mock", derive(Debug))]
#[cfg(any(feature = "testing", test))]
pub struct Featured;
#[cfg(all(unix, feature = "fast"))]
pub struct Fast;
#[cfg(all(test, unix))]
pub struct AllGated;
#[cfg(any(test))]
pub struct AnyGated;
#[cfg(test)]
pub const GATED_CONST: u8 = 1;
#[cfg(test)]
pub enum GatedEnum {}
#[cfg(test)]
pub static GATED_STATIC: u8 = 1;
#[cfg(test)]
pub type GatedType = u8;
#[cfg(test)]
pub union GatedUnion {
    field: u8,
}
#[cfg(test)]
pub use std::fmt;
pub mod open {
    #[cfg(test)]
    pub struct Inner;
}
#[serde(hidden)]
pub struct NotHidden;
#[cfg(target = "test")]
pub struct Targeted;
impl Shown {
    #[cfg(test)]
    pub const GATED: u8 = 1;
    #[cfg(test)]
    pub type Gated = u8;
}
pub fn probe() -> bool {
    cfg!(feature = "test-utils") || cfg!(not(feature = "speed"))
}
