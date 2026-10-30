use meta_cli::{Call, answer};
use meta_ports::real;
use std::io::Read;

fn main() -> real::Exit {
    let stdin = || {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).map_or_else(|_| String::new(), |_| text)
    };
    let reply = match Call::parse(&real::arguments(), stdin) {
        Ok(call) => answer(call, &mut real::ports()),
        Err(usage) => usage,
    };
    print!("{}", reply.text);
    real::exit(reply.code.byte())
}
