#[derive(Debug)]
pub struct Walker;
impl Walker {
    pub fn first<'items>(&self, items: &'items [Walker]) -> Option<&'items Walker> {
        items.iter().for_each(|_| {
            return;
        });
        items.first()
    }
    pub const fn stop(&self) {
        loop {
            break;
        }
    }
    pub fn skip(&self) {
        while true {
            continue;
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn leaves() {
        return;
    }
}
