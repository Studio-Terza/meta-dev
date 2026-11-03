pub struct Order {
    customer: Customer,
}

pub struct Customer {
    orders: Vec<Order>,
}
