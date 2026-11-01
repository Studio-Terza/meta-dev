fn f(x: &str) -> String {
    let t = x.trim();
    let u = String::from("a");
    let v = format!("{t}");
    t.to_string()
}
