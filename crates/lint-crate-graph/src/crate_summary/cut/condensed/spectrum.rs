use std::iter;
use std::iter::Sum;
use std::ops::{Add, Sub};

use super::link::Link;
use super::tangle_index::TangleIndex;
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Weight(f64);

impl Add for Weight {
    type Output = Weight;

    fn add(self, other: Weight) -> Weight {
        Weight(self.0 + other.0)
    }
}

impl Sub for Weight {
    type Output = Weight;

    fn sub(self, other: Weight) -> Weight {
        Weight(self.0 - other.0)
    }
}

impl Sum for Weight {
    fn sum<Weights: Iterator<Item = Weight>>(weights: Weights) -> Weight {
        weights.fold(Weight::default(), Add::add)
    }
}

#[derive(Debug)]
struct Vector(Vec<f64>);

#[derive(Debug)]
pub struct Spectrum<'graph> {
    rows: &'graph [Vec<Link>],
    degrees: Vector,
}

impl<'graph> Spectrum<'graph> {
    const ROUNDS: usize = 300;
    const SEED: f64 = 0.5;
    const STRIDE: f64 = 7.3;
    const OFFSET: f64 = 0.17;

    pub fn new(rows: &'graph [Vec<Link>]) -> Spectrum<'graph> {
        Spectrum {
            rows,
            degrees: Vector(rows.iter().map(|row| row.iter().map(|link| link.weight().into_inner()).sum()).collect()),
        }
    }

    pub fn sweep_order(&self) -> Vec<TangleIndex> {
        let mut order: Vec<(TangleIndex, f64)> =
            self.fiedler().0.into_iter().enumerate().map(|(index, value)| (TangleIndex::from(index), value)).collect();
        order.sort_by(|left, right| left.1.total_cmp(&right.1));
        order.into_iter().map(|(tangle, _)| tangle).collect()
    }

    fn fiedler(&self) -> Vector {
        let top = Vector(self.degrees.0.iter().map(|degree| degree.sqrt()).collect()).unit();
        let start =
            iter::successors(Some(Self::SEED), |seed: &f64| Some(seed.mul_add(Self::STRIDE, Self::OFFSET).fract()))
                .take(self.degrees.0.len());
        let vector = (0..Self::ROUNDS)
            .fold(Vector(start.collect()).deflated(&top), |vector, _| self.lazy_step(&vector).deflated(&top));
        Vector(vector.0.iter().zip(&self.degrees.0).map(|(value, degree)| value / degree.sqrt()).collect())
    }

    fn lazy_step(&self, vector: &Vector) -> Vector {
        let degrees = &self.degrees.0;
        Vector(
            self.rows
                .iter()
                .zip(vector.0.iter().zip(degrees))
                .map(|(row, (value, degree))| {
                    let spread: f64 = row
                        .iter()
                        .filter_map(|link| {
                            let other = link.target().into_inner();
                            Some(
                                link.weight().into_inner() * vector.0.get(other)?
                                    / (degree * degrees.get(other)?).sqrt(),
                            )
                        })
                        .sum();
                    value + spread
                })
                .collect(),
        )
    }
}

impl Vector {
    fn deflated(&self, top: &Vector) -> Vector {
        let along: f64 = self.0.iter().zip(&top.0).map(|(value, unit)| value * unit).sum();
        Vector(self.0.iter().zip(&top.0).map(|(value, unit)| value - along * unit).collect()).unit()
    }

    fn unit(self) -> Vector {
        let length = self.0.iter().map(|value| value * value).sum::<f64>().sqrt();
        Vector(self.0.into_iter().map(|value| value / length).collect())
    }
}
