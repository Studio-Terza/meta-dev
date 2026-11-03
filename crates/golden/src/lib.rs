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

pub use console_ports::Process;
pub use file_ports::Disk;

#[macro_export]
macro_rules! golden {
    ($name:expr, $actual:expr $(,)?) => {
        $crate::Golden::in_package(env!("CARGO_MANIFEST_DIR"), $name)
            .and_then(|golden| golden.verify(&$crate::Process, &$crate::Disk, $crate::Output::new($actual)))
    };
}
