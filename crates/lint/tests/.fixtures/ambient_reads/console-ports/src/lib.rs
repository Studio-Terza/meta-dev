pub struct Inherited;

impl Inherited {
    pub fn variable(name: &str) -> Option<std::ffi::OsString> {
        std::env::var_os(name)
    }
}
