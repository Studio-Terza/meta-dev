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
pub use file_ports::{Disk, Files, Memory};

#[macro_export]
macro_rules! golden {
    ($name:expr, $actual:expr $(,)?) => {
        $crate::verdict!($name, $actual).unwrap_or_else(|failure| panic!("{failure}"))
    };
}

#[macro_export]
macro_rules! verdict {
    ($name:expr, $actual:expr $(,)?) => {
        $crate::Golden::in_package(env!("CARGO_MANIFEST_DIR"), $name)
            .and_then(|golden| golden.verify(&$crate::Process, &$crate::Disk, $crate::Output::new($actual)))
    };
}

#[macro_export]
macro_rules! fixtures {
    ($directory:expr $(,)?) => {{
        let mirrored = $crate::Memory::default();
        $crate::Files::mirror(
            &$crate::Disk,
            &::std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join($directory),
            &mirrored,
        )
        .map(|()| mirrored)
    }};
}
