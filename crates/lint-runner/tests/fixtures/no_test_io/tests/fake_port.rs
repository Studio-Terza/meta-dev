use file_ports::{Files, Memory};
use std::path::Path;
#[test]
fn reads() {
    Memory::default().read(Path::new("a")).ok();
}
