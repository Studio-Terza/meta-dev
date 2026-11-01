fn f(items: Vec<u8>, out: &mut Vec<u8>) {
    for item in items {
        if item > 1 {
            out.push(item);
        }
    }
}
