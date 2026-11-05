use lint_core::{Headline, Violation};

use super::cluster::Level;
use super::consumed::Interface;
use super::{ClusterLint, Containment};

#[derive(Debug)]
pub struct InterfaceByConsumers;

impl ClusterLint for InterfaceByConsumers {
    const HEADLINE: Headline = Headline::new(
        "each module has one interface by its users: its exports are the types and traits used from outside it, two exports are linked when one outside type or trait uses both, when one names the other in its fields, variants, bounds or method signatures (parameters, returns and errors), or when one implements a trait for the other, and the linked exports form one group",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.splits::<Interface<'_, '_>>(Level::Module)
    }
}
