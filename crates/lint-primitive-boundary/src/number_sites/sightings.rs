use lint_core::Violation;

use super::sighting::Sighting;

#[derive(Default)]
pub struct Sightings(Vec<Sighting>);

impl Sightings {
    pub fn file(&mut self, sighting: Option<Sighting>) {
        self.0.extend(sighting);
    }

    pub fn into_violations(self) -> Vec<Violation> {
        let mut sightings = self.0;
        sightings.sort_by_key(Sighting::rank);
        sightings.into_iter().map(Sighting::into_violation).collect()
    }
}
