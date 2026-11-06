use std::fmt::Display;
use std::str::FromStr;

use design_model::{
    Confidence, Entered, Explicitness, Facet, Kin, NodeId, NodeRelation, Project, Provenance, RunId, SegmentRelation,
    SenseKind, SpanRelation, Specificity, Stage,
};
use laws::proptest::prelude::{prop_assert, prop_assert_eq};
use laws::proptest::strategy::Strategy;
use strum::VariantArray;

struct Names;

impl Names {
    fn unparsed<Named>(name: impl Fn(Named) -> &'static str) -> Vec<Named>
    where
        Named: VariantArray + Copy + FromStr + PartialEq,
    {
        Named::VARIANTS
            .iter()
            .copied()
            .filter(|variant| name(*variant).parse::<Named>().ok() != Some(*variant))
            .collect()
    }

    fn undisplayed<Named>(name: impl Fn(Named) -> &'static str) -> Vec<Named>
    where
        Named: VariantArray + Copy + Display,
    {
        Named::VARIANTS.iter().copied().filter(|variant| variant.to_string() != name(*variant)).collect()
    }
}

struct Projects;

impl Projects {
    fn any() -> impl Strategy<Value = Project> {
        "[A-Za-z0-9][A-Za-z0-9._-]{0,15}".prop_filter_map("a project name", |name| name.parse::<Project>().ok())
    }
}

#[test]
fn every_facet_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Facet::name), []);
    assert_eq!(Names::undisplayed(Facet::name), []);
}

#[test]
fn every_stage_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Stage::name), []);
    assert_eq!(Names::undisplayed(Stage::name), []);
}

#[test]
fn every_node_relation_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(NodeRelation::name), []);
    assert_eq!(Names::undisplayed(NodeRelation::name), []);
}

#[test]
fn every_kin_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Kin::name), []);
    assert_eq!(Names::undisplayed(Kin::name), []);
}

#[test]
fn every_span_relation_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(SpanRelation::name), []);
    assert_eq!(Names::undisplayed(SpanRelation::name), []);
}

#[test]
fn every_segment_relation_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(SegmentRelation::name), []);
    assert_eq!(Names::undisplayed(SegmentRelation::name), []);
}

#[test]
fn every_sense_kind_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(SenseKind::name), []);
    assert_eq!(Names::undisplayed(SenseKind::name), []);
}

#[test]
fn every_specificity_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Specificity::name), []);
    assert_eq!(Names::undisplayed(Specificity::name), []);
}

#[test]
fn every_confidence_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Confidence::name), []);
    assert_eq!(Names::undisplayed(Confidence::name), []);
}

#[test]
fn every_explicitness_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Explicitness::name), []);
    assert_eq!(Names::undisplayed(Explicitness::name), []);
}

#[test]
fn every_way_of_entering_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Entered::name), []);
    assert_eq!(Names::undisplayed(Entered::name), []);
}

#[test]
fn every_way_of_entering_survives_becoming_a_provenance_and_its_text() {
    let recovered: Vec<Option<Provenance>> =
        Entered::VARIANTS.iter().map(|entered| Provenance::from(*entered).to_string().parse().ok()).collect();
    let expected: Vec<Option<Provenance>> =
        Entered::VARIANTS.iter().map(|entered| Some(Provenance::from(*entered))).collect();
    assert_eq!(recovered, expected);
}

laws::roundtrip!(
    an_agent_provenance_is_displayed_and_parsed_back,
    laws::proptest::arbitrary::any::<RunId>(),
    |run: &RunId| Provenance::Agent(*run).to_string(),
    |text: &String| text.parse::<Provenance>().ok(),
    |run: RunId| Some(Provenance::Agent(run))
);

laws::law!(a_provenance_never_reads_from_other_words, "[a-z]{0,8}( [a-z]{1,3})?", text, {
    prop_assert!(text.parse::<Provenance>().ok().is_none_or(|provenance| matches!(provenance, Provenance::Lead(_))));
});

#[test]
fn a_word_naming_no_variant_is_refused() {
    assert!("Project".parse::<Facet>().is_err());
    assert!("".parse::<NodeRelation>().is_err());
}

laws::roundtrip!(
    a_project_is_displayed_and_parsed_back,
    Projects::any(),
    |project: &Project| project.to_string(),
    |text: &String| text.parse::<Project>().ok(),
    Some
);

laws::roundtrip!(
    a_project_is_serialized_and_deserialized_back,
    Projects::any(),
    |project: &Project| serde_json::to_string(project).ok(),
    |text: &Option<String>| text.as_deref().and_then(|text| serde_json::from_str::<Project>(text).ok()),
    Some
);

laws::law!(a_project_name_never_starts_or_holds_what_names_cannot, "[ ./_-][a-z]{0,6}|[a-z]{1,6} [a-z]{1,6}|", text, {
    prop_assert!(text.parse::<Project>().is_err());
    let quoted = format!("{text:?}");
    prop_assert!(serde_json::from_str::<Project>(&quoted).is_err());
});

laws::law!(a_node_id_is_read_from_n_and_its_number, 0..i64::MAX, number, {
    let spelled = format!("n{number}");
    prop_assert_eq!(spelled.parse::<NodeId>().ok(), Some(NodeId::new(number)));
});

laws::law!(no_message_uuid_reads_as_a_node_id, "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", uuid, {
    prop_assert!(uuid.parse::<NodeId>().is_err());
});

laws::law!(text_beyond_n_and_digits_is_not_a_node_id, ("[0-9]{1,6}", "[^0-9]{1,3}"), parts, {
    let (digits, rest) = parts;
    let spelled = format!("n{digits}{rest}");
    prop_assert!(spelled.parse::<NodeId>().is_err());
    prop_assert!(digits.parse::<NodeId>().is_err());
});
