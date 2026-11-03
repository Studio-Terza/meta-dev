mod ports;

use std::fs;

#[derive(Debug)]
pub struct Notes;

impl Notes {
    pub fn read() {
        fs::read_to_string("notes.txt").ok();
        std::env::var_os("HOME");
        println!("read");
    }
}
