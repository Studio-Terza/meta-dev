use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Product {
    Equal,
    Below,
    Above,
    Apart,
}

impl Product {
    const MEETS: [(Product, Product, Product); 7] = [
        (Product::Equal, Product::Equal, Product::Equal),
        (Product::Equal, Product::Below, Product::Below),
        (Product::Equal, Product::Above, Product::Above),
        (Product::Below, Product::Equal, Product::Below),
        (Product::Below, Product::Below, Product::Below),
        (Product::Above, Product::Equal, Product::Above),
        (Product::Above, Product::Above, Product::Above),
    ];

    pub(crate) fn combined(orders: impl IntoIterator<Item = Option<Ordering>>) -> Option<Ordering> {
        orders.into_iter().map(Product::from).fold(Product::Equal, Product::met).into()
    }

    fn met(self, next: Product) -> Product {
        Self::MEETS
            .iter()
            .filter(|(mine, _, _)| *mine == self)
            .find(|(_, theirs, _)| *theirs == next)
            .map_or(Product::Apart, |(_, _, met)| *met)
    }
}

impl From<Option<Ordering>> for Product {
    fn from(order: Option<Ordering>) -> Product {
        match order {
            Some(Ordering::Equal) => Product::Equal,
            Some(Ordering::Less) => Product::Below,
            Some(Ordering::Greater) => Product::Above,
            None => Product::Apart,
        }
    }
}

impl From<Product> for Option<Ordering> {
    fn from(product: Product) -> Option<Ordering> {
        match product {
            Product::Equal => Some(Ordering::Equal),
            Product::Below => Some(Ordering::Less),
            Product::Above => Some(Ordering::Greater),
            Product::Apart => None,
        }
    }
}
