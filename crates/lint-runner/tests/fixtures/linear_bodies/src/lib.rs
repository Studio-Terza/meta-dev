pub struct Gauge;

impl Gauge {
    pub fn sized(&self, items: Option<u8>) -> u8 {
        let count = match items {
            Some(count) => count,
            None => 0,
        };
        if count > 9 { 9 } else { count }
    }
}
