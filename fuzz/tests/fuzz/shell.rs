#[derive(Clone, Copy, Debug)]
pub struct Shell {
    open: &'static str,
    close: &'static str,
}

impl Shell {
    pub const fn new(open: &'static str, close: &'static str) -> Shell {
        Shell { open, close }
    }

    pub fn around(self, core: &str, depth: usize) -> String {
        format!("{}{core}{}", self.open.repeat(depth), self.close.repeat(depth))
    }
}
