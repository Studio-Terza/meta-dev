use strum::VariantArray;

pub trait Enumerated: Sized + 'static {
    const VARIANTS: &'static [Self];
}

impl<Listed: VariantArray> Enumerated for Listed {
    const VARIANTS: &'static [Listed] = <Listed as VariantArray>::VARIANTS;
}
