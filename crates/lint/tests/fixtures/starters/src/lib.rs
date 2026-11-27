use std::process;
use std::process::{self as spawning};
use std::process as launcher;
use std::fmt as other;
use core::process as alt;
fn aliased() {
    launcher::Command::new("ls");
    other::Command::new("ls");
    alt::Command::new("ls");
}
mod globbed {
    use std::process::*;
    fn globbed() {
        Command::new("ls");
        run!(Command::new("ls"));
        run!(Command::new::<u8>("ls"));
    }
}
mod ports {
    fn ported() {
        std::process::Command::new("ls");
    }
    mod deeper {
        fn deeper() {
            std::process::Command::new("ls");
        }
    }
}
mod unimported {
    fn unimported() {
        Command::new("ls");
        Other::new("ls");
        process::Command::spawn("ls");
    }
}
fn module() {
    process::Command::new("ls");
}
fn renamed() {
    spawning::Command::new("ls");
}
fn spoken() {
    run!(std::process::Command::new("ls"), process::Command::new, std::process::Command::);
}
fn typed(command: Vec<std::process::Command>) {}
