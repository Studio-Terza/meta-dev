pub struct Gauge;

impl Gauge {
    pub fn sized(&self, items: Option<u8>) -> u8 {
        match items {
            Some(9) => 9,
            Some(count) if count > 9 => {
                let capped = count;
                capped
            }
            Some(count) => count,
            None => 0,
        }
    }
}
