use std::io::{self, Write};

#[derive(Debug)]
pub struct Tee<First, Second> {
    first: First,
    second: Second,
}

impl<First: Write, Second: Write> Tee<First, Second> {
    pub fn new(first: First, second: Second) -> Tee<First, Second> {
        Tee { first, second }
    }
}

impl<First: Write, Second: Write> Write for Tee<First, Second> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.first.write_all(bytes)?;
        self.second.write_all(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.first.flush()?;
        self.second.flush()
    }
}
