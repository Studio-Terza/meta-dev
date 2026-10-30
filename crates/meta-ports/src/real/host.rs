use meta_core::Build;
pub use std::process::ExitCode as Exit;

pub fn arguments() -> Vec<String> {
    std::env::args().skip(1).collect()
}

/// The commit the build script embedded (`build.rs`), or why it embedded none.
pub fn build() -> Build {
    Build::embedded(
        option_env!("META_BUILD_COMMIT"),
        option_env!("META_BUILD_DIRTY"),
        option_env!("META_BUILD_UNVERIFIED"),
    )
}

pub fn exit(code: u8) -> Exit {
    Exit::from(code)
}
