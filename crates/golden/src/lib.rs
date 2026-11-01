mod difference;
mod failure;
mod golden;
mod mode;

pub use difference::Difference;
pub use failure::Failure;
pub use golden::Golden;
pub use mode::Mode;

#[macro_export]
macro_rules! golden {
    ($name:expr, $actual:expr $(,)?) => {
        $crate::Golden::in_package(env!("CARGO_MANIFEST_DIR"), $name).verify($actual)
    };
}
