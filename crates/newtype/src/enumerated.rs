pub trait Enumerated: Sized + 'static {
    const VARIANTS: &'static [Self];
}
