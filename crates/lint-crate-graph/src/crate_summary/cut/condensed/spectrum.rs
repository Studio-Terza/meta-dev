use std::iter;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Weight(f64);

impl Weight {
    pub fn new(weight: f64) -> Weight {
        Weight(weight)
    }

    pub fn crossings(self) -> f64 {
        self.0
    }
}

#[derive(Debug)]
pub struct Spectrum<'graph> {
    weights: &'graph [Vec<Weight>],
    degrees: Vec<f64>,
}

impl<'graph> Spectrum<'graph> {
    const ROUNDS: usize = 300;
    const SEED: f64 = 0.5;
    const STRIDE: f64 = 7.3;
    const OFFSET: f64 = 0.17;

    pub fn new(weights: &'graph [Vec<Weight>]) -> Spectrum<'graph> {
        Spectrum { weights, degrees: weights.iter().map(|row| row.iter().map(|weight| weight.0).sum()).collect() }
    }

    pub fn fiedler(&self) -> Vec<f64> {
        let top = Self::unit(self.degrees.iter().map(|degree| degree.sqrt()).collect());
        let start = iter::successors(Some(Self::SEED), |seed: &f64| Some((seed * Self::STRIDE + Self::OFFSET).fract()))
            .take(self.degrees.len());
        let vector = (0..Self::ROUNDS).fold(Self::deflated(&start.collect::<Vec<f64>>(), &top), |vector, _| {
            Self::deflated(&self.lazy_step(&vector), &top)
        });
        vector.iter().zip(&self.degrees).map(|(value, degree)| value / degree.sqrt()).collect()
    }

    fn lazy_step(&self, vector: &[f64]) -> Vec<f64> {
        self.weights
            .iter()
            .zip(vector.iter().zip(&self.degrees))
            .map(|(weights, (value, degree))| {
                let spread: f64 = weights
                    .iter()
                    .zip(vector.iter().zip(&self.degrees))
                    .map(|(weight, (other, other_degree))| weight.0 * other / (degree * other_degree).sqrt())
                    .sum();
                value + spread
            })
            .collect()
    }

    fn deflated(vector: &[f64], top: &[f64]) -> Vec<f64> {
        let along: f64 = vector.iter().zip(top).map(|(value, unit)| value * unit).sum();
        Self::unit(vector.iter().zip(top).map(|(value, unit)| value - along * unit).collect())
    }

    fn unit(vector: Vec<f64>) -> Vec<f64> {
        let length = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
        vector.into_iter().map(|value| value / length).collect()
    }
}
