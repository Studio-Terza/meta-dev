#[derive(Debug)]
pub struct Disk;

impl Disk {
    pub fn write() {
        std::fs::write("notes.txt", "notes").ok();
    }
}
