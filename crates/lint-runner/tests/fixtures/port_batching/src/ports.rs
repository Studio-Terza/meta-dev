pub struct Id;

pub trait Store {
    fn fetch(&self, id: &Id) -> Id;

    fn fetch_many(&self, ids: &[Id]) -> Vec<Id>;
}
