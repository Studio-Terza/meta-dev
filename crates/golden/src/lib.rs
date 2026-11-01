mod difference;
mod failure;
mod golden;
mod mode;
mod output;

pub use difference::Difference;
pub use failure::Failure;
pub use golden::Golden;
pub use mode::Mode;
pub use output::Output;

#[macro_export]
macro_rules! golden {
    ($name:expr, $actual:expr $(,)?) => {
        $crate::Golden::in_running_package($name).and_then(|golden| golden.verify($crate::Output::new($actual)))
    };
}
