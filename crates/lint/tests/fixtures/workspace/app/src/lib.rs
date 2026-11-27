pub mod nested;
mod flat;
mod inline {
    mod deep;
    #[path = "beside.rs"]
    mod beside;
}
#[path = "elsewhere/placed.rs"]
mod placed;
#[cfg(test)]
mod checks;
mod again;
#[cfg(test)]
mod inline_tests {
    #[test]
    fn inline() {}
}
