struct A { count: u8 }
fn f() -> Result<u8, std::fmt::Error> { Ok(1) }
fn g() {}
#[cfg(test)]
mod tests {
    struct B { on: bool }
}
