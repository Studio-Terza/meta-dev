use std::fmt;

use graph::Model;
use lint_core::{Fix, Site};

use super::ReadKey;

#[derive(Debug)]
pub struct Finding<'model> {
    model: &'model Model,
    key: ReadKey<'model>,
    sites: Vec<Site<'model>>,
}

impl<'model> Finding<'model> {
    pub fn new(model: &'model Model, key: ReadKey<'model>, sites: Vec<Site<'model>>) -> Finding<'model> {
        Finding { model, key, sites }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sites: Vec<String> = self.sites.iter().map(ToString::to_string).collect();
        write!(formatter, "{}; at {}", self.key.spelled(self.model), sites.join(" "))
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new("compute it once where the method is called and pass it in")
    }
}
