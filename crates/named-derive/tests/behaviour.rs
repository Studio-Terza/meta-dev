use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Snaked {
    Open,
    RepliesTo,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
#[named(kebab)]
enum Kebabbed {
    Open,
    ConsumedBy,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Overridden {
    #[name("Mon")]
    Monday,
    Tuesday,
}

laws::table!(a_variant_without_a_name_is_spelled_in_snake_case, |state: &Snaked| state.to_string(), [
    Snaked::Open => "open",
    Snaked::RepliesTo => "replies_to",
]);

laws::table!(a_kebab_enum_spells_its_variants_in_kebab_case, |relation: &Kebabbed| relation.to_string(), [
    Kebabbed::Open => "open",
    Kebabbed::ConsumedBy => "consumed-by",
]);

laws::table!(an_explicit_name_overrides_the_default, |day: &Overridden| day.to_string(), [
    Overridden::Monday => "Mon",
    Overridden::Tuesday => "tuesday",
]);

laws::table!(each_spelling_reads_back_as_its_variant, |text: &&str| text.parse::<Kebabbed>().map_err(|unknown| unknown.to_string()), [
    "consumed-by" => Ok(Kebabbed::ConsumedBy),
    "consumed_by" => Err("\"consumed_by\" names no Kebabbed".to_owned()),
]);
