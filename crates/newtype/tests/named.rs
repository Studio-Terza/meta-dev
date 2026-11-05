use newtype::{Named, Unnamed};
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
enum Lamp {
    #[name("on")]
    On,
    #[name("off")]
    Off,
    #[name("dimmed-low")]
    Dimmed,
}

struct Names;

impl Names {
    const LIT: &'static str = Lamp::On.name();

    fn parsed(text: &&str) -> Result<Lamp, Unnamed<Lamp>> {
        text.parse()
    }

    fn refused(text: &&str) -> Option<String> {
        text.parse::<Lamp>().err().map(|refusal| refusal.to_string())
    }
}

laws::table!(each_variant_has_its_declared_name, |lamp: &Lamp| lamp.name(), [Lamp::On => "on", Lamp::Off => "off", Lamp::Dimmed => "dimmed-low"]);

laws::table!(display_writes_the_declared_name, |lamp: &Lamp| lamp.to_string(), [Lamp::On => "on", Lamp::Dimmed => "dimmed-low"]);

laws::table!(display_honours_width_and_alignment, |lamp: &Lamp| (format!("{lamp:>5}"), format!("{lamp:<5}|"), format!("{lamp:^7}")), [
    Lamp::On => (String::from("   on"), String::from("on   |"), String::from("  on   ")),
]);

laws::table!(from_str_reads_each_name_and_refuses_any_other_text, Names::parsed, [
    "off" => Ok(Lamp::Off),
    "dimmed-low" => Ok(Lamp::Dimmed),
    "On" => Err(Unnamed::new("On")),
    "" => Err(Unnamed::new("")),
]);

laws::table!(a_refusal_names_the_text_and_every_name, Names::refused, ["dim" => Some(String::from("`dim` is not one of: on, off, dimmed-low")), "on" => None]);

laws::table!(every_variant_parses_back_from_its_name, |lamp: &Lamp| lamp.name().parse::<Lamp>(), [Lamp::On => Ok(Lamp::On), Lamp::Off => Ok(Lamp::Off), Lamp::Dimmed => Ok(Lamp::Dimmed)]);

#[test]
fn a_name_is_a_compile_time_constant() {
    assert_eq!(Names::LIT, "on");
    assert_eq!(Lamp::VARIANTS.iter().map(|lamp| lamp.name()).collect::<Vec<_>>(), ["on", "off", "dimmed-low"]);
}

#[derive(Named, VariantArray, Clone, Copy)]
enum Bare {
    #[name("bare")]
    Only,
}

impl Bare {
    fn read(text: &str) -> Result<Bare, Box<dyn std::error::Error>> {
        Ok(text.parse::<Bare>()?)
    }
}

#[test]
fn a_refusal_boxes_into_dyn_error_for_an_enum_without_debug_or_partial_eq() {
    assert_eq!(
        Bare::read("other").err().map(|error| error.to_string()),
        Some("`other` is not one of: bare".to_owned())
    );
    assert!(Bare::read("bare").is_ok());
}

#[derive(Named, VariantArray, Clone, Copy)]
#[accessor(glyph: &'static str)]
#[accessor(rank: u8)]
#[accessor(lit: bool)]
#[accessor(mark: char)]
enum Lit {
    #[name("hi")]
    #[value(glyph = "*")]
    #[value(rank = 7)]
    #[value(lit = true)]
    #[value(mark = 'h')]
    High,
    #[name("lo")]
    #[value(mark = 'l')]
    #[value(lit = false)]
    #[value(rank = 1)]
    #[value(glyph = "-")]
    Low,
}

struct Accessors;

impl Accessors {
    const GLYPH: &'static str = Lit::High.glyph();
    const RANK: u8 = Lit::Low.rank();
    const LIT: bool = Lit::High.lit();
    const MARK: char = Lit::Low.mark();
}

laws::table!(accessors_read_each_variants_values, |lit: &Lit| (lit.glyph(), lit.rank(), lit.lit(), lit.mark()), [
    Lit::High => ("*", 7, true, 'h'),
    Lit::Low => ("-", 1, false, 'l'),
]);

#[test]
fn accessors_are_compile_time_constants() {
    assert_eq!(Accessors::GLYPH, "*");
    assert_eq!(Accessors::RANK, 1);
    assert_eq!(u8::from(Accessors::LIT), 1);
    assert_eq!(Accessors::MARK, 'l');
}
