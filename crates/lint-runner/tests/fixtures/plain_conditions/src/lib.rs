pub struct Gauge;

impl Gauge {
    pub fn kept(&self, items: Vec<u8>) -> Vec<u8> {
        items.into_iter().filter(|item| *item > 1 && *item < 9).filter(|item| !(*item == 4 || *item == 5)).collect()
    }
}
