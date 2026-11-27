pub struct Raw {
    text: String,
}
pub fn parsed(text: &str) -> Option<u32> {
    text.parse().ok()
}
