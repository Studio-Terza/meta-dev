struct Ctx { a: u8 }
fn fact(c: &Ctx) -> u8 { c.a }
trait Lint { fn run(&self, c: &Ctx) -> u8; }
struct A;
struct B;
impl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }
impl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) + 2 } }
fn all(lints: &[&dyn Lint]) -> usize { lints.len() }
