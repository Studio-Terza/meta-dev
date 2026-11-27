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
pub fn probe() -> bool {
    cfg!(feature = "test-utils") || cfg!(not(feature = "speed"))
}
