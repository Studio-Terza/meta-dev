use file_ports::{Disk, Files};
use std::path::Path;
#[test]
fn reads() {
    Disk.read(Path::new("a")).ok();
}
