#[derive(Debug)]
pub struct Regrouped;

impl Regrouped {
    #[must_use]
    pub fn like<Shape, Item>(shape: &[Vec<Shape>], flat: Vec<Item>) -> Vec<Vec<Item>> {
        shape
            .iter()
            .map(Vec::len)
            .scan(flat.into_iter(), |rest, length| Some(rest.by_ref().take(length).collect()))
            .collect()
    }
}
