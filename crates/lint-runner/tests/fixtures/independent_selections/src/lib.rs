pub struct Shelf;

impl Shelf {
    pub fn lit(&self, items: &[Lamp], ready: Ready) -> Vec<Lamp> {
        items.iter().copied().filter(|lamp| lamp.on()).filter(|_| ready.now()).collect()
    }

    pub fn first(&self, items: &[Lamp], ready: Ready) -> Option<Lamp> {
        items.first().copied().filter(|_| ready.now())
    }

    pub fn found(&self, items: &[Lamp], ready: Ready) -> Option<usize> {
        items.iter().position(|lamp| matches!(lamp, Lamp::Dim)).or(items.iter().position(|_| ready.now()))
    }

    pub fn every(&self, items: &[Lamp]) -> Lamps {
        Lamps::from(items.iter().all(|&Lamp { on }| on))
    }
}
