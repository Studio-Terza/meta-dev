#![deny(clippy::wildcard_enum_match_arm)]

use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use design_model::{
    AnyNode, AnyStage, Artifact, ArtifactKind, Basis, Condition, Confidence, Dropped, Entered, Entry, Explicitness,
    Facet, Inversion, Kin, Misstaged, Node, NodeEdge, NodeId, NodeRelation, NotesLine, Project, Provenance, Reason,
    Role, RunId, SegmentRelation, SenseKind, SpanRelation, Specificity, Stage, Terminal, Title, Transition,
};
use laws::proptest::collection::vec;
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq, prop_oneof};
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
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
fn every_basis_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Basis::name), []);
    assert_eq!(Names::undisplayed(Basis::name), []);
}

laws::law!(
    a_set_basis_keeps_the_stored_stage_a_decided_one_refines_and_a_vague_one_stays_vague,
    select(Stage::VARIANTS),
    stored,
    {
        let staged: Vec<Stage> = Basis::VARIANTS.iter().map(|basis| basis.staged(stored)).collect();
        prop_assert_eq!(staged, [stored, Stage::Refining, Stage::Vague]);
    }
);

#[test]
fn every_role_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(Role::name), []);
    assert_eq!(Names::undisplayed(Role::name), []);
}

#[test]
fn every_artifact_kind_is_displayed_as_its_name_and_parsed_back_from_it() {
    assert_eq!(Names::unparsed(ArtifactKind::name), []);
    assert_eq!(Names::undisplayed(ArtifactKind::name), []);
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

struct Provenances;

impl Provenances {
    fn any() -> impl Strategy<Value = Provenance> {
        prop_oneof![
            select(Entered::VARIANTS).prop_map(Provenance::from),
            any::<i64>().prop_map(|run| Provenance::Agent(RunId::new(run))),
            (1..=usize::MAX).prop_map(|line| Provenance::Import(NotesLine::new(line))),
        ]
    }
}

laws::roundtrip!(
    a_provenance_is_displayed_and_parsed_back,
    Provenances::any(),
    |provenance: &Provenance| provenance.to_string(),
    |text: &String| text.parse::<Provenance>().ok(),
    Some
);

laws::table!(
    a_provenance_is_a_way_of_entering_an_agent_run_or_a_notes_line,
    |text: &&'static str| text.parse::<Provenance>().ok(),
    [
        "apply" => Some(Provenance::from(Entered::Apply)),
        "stage" => Some(Provenance::from(Entered::Stage)),
        "agent-7" => Some(Provenance::Agent(RunId::new(7))),
        "agent-0" => Some(Provenance::Agent(RunId::new(0))),
        "agent--3" => Some(Provenance::Agent(RunId::new(-3))),
        "agent" => None,
        "agent-" => None,
        "agent-07" => None,
        "agent-7 " => None,
        "agent 7" => None,
        "agent-99999999999999999999" => None,
        "import-12" => Some(Provenance::Import(NotesLine::new(12))),
        "import-1" => Some(Provenance::Import(NotesLine::new(1))),
        "import-0" => None,
        "import-012" => None,
        "import--1" => None,
        "import" => None,
        "import-99999999999999999999999" => None,
    ]
);

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
    |text: &Option<String>| text.as_deref().and_then(|written| serde_json::from_str::<Project>(written).ok()),
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

#[derive(Clone, Copy, Debug)]
struct Asked(Stage, Move);

#[derive(Clone, Copy, Debug)]
struct Ends(NodeRelation, Role, Role);

#[derive(Clone, Copy, Debug)]
enum Move {
    Refine,
    Specify,
    Reopen,
    Drop,
}

struct Lifecycle;

impl Lifecycle {
    fn transition(step: Move) -> Result<Transition, Box<dyn Error>> {
        match step {
            Move::Refine => Ok(Transition::Refine),
            Move::Specify => Ok(Transition::Specify {
                artifact: "lint:no-comments".parse()?,
                condition: "every comment is refused".parse()?,
            }),
            Move::Reopen => Ok(Transition::Reopen),
            Move::Drop => Ok(Transition::Drop { reason: "folded into its parent".parse()? }),
        }
    }

    fn fresh() -> Result<AnyNode, Box<dyn Error>> {
        let entry = Entry::new("demo".parse()?, Facet::Feature, Provenance::from(Entered::Apply));
        Ok(Node::new(NodeId::new(7), entry).titled(Title::new("lifecycle".to_owned())).any())
    }

    fn path(stage: Stage) -> &'static [Move] {
        match stage {
            Stage::Vague => &[],
            Stage::Refining => &[Move::Refine],
            Stage::Terminal => &[Move::Refine, Move::Specify],
            Stage::Dropped => &[Move::Drop],
        }
    }

    fn reached(stage: Stage) -> Result<AnyNode, Box<dyn Error>> {
        Lifecycle::path(stage)
            .iter()
            .try_fold(Lifecycle::fresh()?, |node, step| Ok(Lifecycle::transition(*step)?.applied(node)?))
    }

    fn moved(Asked(stage, step): Asked) -> Result<Stage, Stage> {
        let node = Lifecycle::reached(stage).map_err(|_unreached| stage)?;
        let transition = Lifecycle::transition(step).map_err(|_unparsed| stage)?;
        transition.applied(node).map(|moved| moved.stage().stage()).map_err(Misstaged::stage)
    }
}

laws::table!(
    each_transition_moves_only_from_its_allowed_stages_and_is_refused_naming_the_actual_stage_elsewhere,
    |asked: &Asked| Lifecycle::moved(*asked),
    [
        Asked(Stage::Vague, Move::Refine) => Ok(Stage::Refining),
        Asked(Stage::Refining, Move::Refine) => Err(Stage::Refining),
        Asked(Stage::Terminal, Move::Refine) => Err(Stage::Terminal),
        Asked(Stage::Dropped, Move::Refine) => Err(Stage::Dropped),
        Asked(Stage::Vague, Move::Specify) => Err(Stage::Vague),
        Asked(Stage::Refining, Move::Specify) => Ok(Stage::Terminal),
        Asked(Stage::Terminal, Move::Specify) => Err(Stage::Terminal),
        Asked(Stage::Dropped, Move::Specify) => Err(Stage::Dropped),
        Asked(Stage::Vague, Move::Reopen) => Err(Stage::Vague),
        Asked(Stage::Refining, Move::Reopen) => Err(Stage::Refining),
        Asked(Stage::Terminal, Move::Reopen) => Ok(Stage::Refining),
        Asked(Stage::Dropped, Move::Reopen) => Ok(Stage::Refining),
        Asked(Stage::Vague, Move::Drop) => Ok(Stage::Dropped),
        Asked(Stage::Refining, Move::Drop) => Ok(Stage::Dropped),
        Asked(Stage::Terminal, Move::Drop) => Ok(Stage::Dropped),
        Asked(Stage::Dropped, Move::Drop) => Err(Stage::Dropped),
    ]
);

#[test]
fn a_refused_transition_names_the_node_it_was_asked_of() -> Result<(), Box<dyn Error>> {
    let refused = Transition::Reopen.applied(Lifecycle::fresh()?).err().map(Misstaged::node);
    assert_eq!(refused, Some(NodeId::new(7)));
    Ok(())
}

#[test]
fn a_specified_node_holds_its_artifact_and_condition_and_a_dropped_one_its_reason() -> Result<(), Box<dyn Error>> {
    let artifact: Artifact = "lint:no-comments".parse()?;
    let condition: Condition = "every comment is refused".parse()?;
    let reason: Reason = "folded into its parent".parse()?;
    let terminal = Lifecycle::reached(Stage::Terminal)?;
    let dropped = Lifecycle::reached(Stage::Dropped)?;
    assert_eq!(terminal.stage(), &AnyStage::from(Terminal::new(artifact, condition)));
    assert_eq!(dropped.stage(), &AnyStage::from(Dropped::from(reason)));
    Ok(())
}

laws::law!(
    a_node_keeps_its_identity_entry_title_and_role_through_every_transition,
    vec(select(&[Move::Refine, Move::Specify, Move::Reopen, Move::Drop][..]), 0..8),
    steps,
    {
        let fresh = Lifecycle::fresh().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let walked = steps.iter().try_fold(fresh.clone(), |node, step| {
            let transition =
                Lifecycle::transition(*step).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
            Ok::<AnyNode, TestCaseError>(transition.applied(node.clone()).unwrap_or(node))
        })?;
        prop_assert_eq!(walked.identity(), fresh.identity());
        prop_assert_eq!(walked.entry(), fresh.entry());
        prop_assert_eq!(walked.title(), fresh.title());
        prop_assert_eq!(walked.casting().role(), Role::Core);
    }
);

laws::roundtrip!(
    an_artifact_is_displayed_and_parsed_back,
    (select(ArtifactKind::VARIANTS), "[A-Za-z0-9_][A-Za-z0-9_.:/-]{0,12}"),
    |(kind, name): &(ArtifactKind, String)| format!("{kind}:{name}"),
    |text: &String| text.parse::<Artifact>().ok().map(|artifact| artifact.to_string()),
    |(kind, name): (ArtifactKind, String)| Some(format!("{kind}:{name}"))
);

laws::table!(
    an_artifact_needs_a_known_kind_a_colon_and_a_name,
    |text: &&str| text.parse::<Artifact>().ok().map(|artifact| artifact.to_string()),
    [
        "behaviour:a_dropped_node_stays_resolvable" => Some("behaviour:a_dropped_node_stays_resolvable".to_owned()),
        "hook:stop" => Some("hook:stop".to_owned()),
        "rule:no-comments" => None,
        "Lint:no-comments" => None,
        "lint:" => None,
        "lint" => None,
        ":no-comments" => None,
        "lint:no comments" => None,
        "lint: no-comments" => None,
        "" => None,
    ]
);

laws::roundtrip!(
    a_condition_and_a_reason_are_displayed_and_parsed_back,
    "[ -~]{0,4}[!-~][ -~]{0,20}",
    |text: &String| text.clone(),
    |text: &String| {
        let condition = text.parse::<Condition>().ok().map(|condition| condition.to_string());
        let reason = text.parse::<Reason>().ok().map(|reason| reason.to_string());
        [condition, reason]
    },
    |text: String| [Some(text.clone()), Some(text)]
);

laws::law!(
    an_empty_blank_or_broken_line_is_neither_a_condition_nor_a_reason,
    "[ \t]{0,4}|[a-z ]{0,6}[\r\n\x0B\x0C\u{85}\u{2028}\u{2029}][a-z ]{0,6}",
    text,
    {
        prop_assert!(text.parse::<Condition>().is_err());
        prop_assert!(text.parse::<Reason>().is_err());
    }
);

struct Dependence;

impl Dependence {
    fn upheld(Ends(relation, source, target): Ends) -> Option<NodeEdge> {
        let edge = NodeEdge::new(NodeId::new(2), relation, NodeId::new(1));
        edge.upheld(source, target).err().map(Inversion::edge)
    }
}

laws::table!(
    a_core_node_never_depends_on_a_secondary_one_and_the_refusal_names_the_edge,
    |ends: &Ends| Dependence::upheld(*ends),
    [
        Ends(NodeRelation::Refines, Role::Core, Role::Secondary) => Some(NodeEdge::new(NodeId::new(2), NodeRelation::Refines, NodeId::new(1))),
        Ends(NodeRelation::Refines, Role::Secondary, Role::Core) => None,
        Ends(NodeRelation::Refines, Role::Core, Role::Core) => None,
        Ends(NodeRelation::Refines, Role::Secondary, Role::Secondary) => None,
        Ends(NodeRelation::Consumes, Role::Core, Role::Secondary) => Some(NodeEdge::new(NodeId::new(2), NodeRelation::Consumes, NodeId::new(1))),
        Ends(NodeRelation::Consumes, Role::Secondary, Role::Core) => None,
        Ends(NodeRelation::Supersedes, Role::Core, Role::Secondary) => None,
        Ends(NodeRelation::Supersedes, Role::Secondary, Role::Core) => None,
    ]
);

#[test]
fn a_refused_provenance_lists_every_way_of_entering_and_nothing_else_before_the_agent_and_import_forms() {
    let leads = Entered::VARIANTS.iter().map(ToString::to_string).collect::<Vec<String>>().join(", ");
    let refusal = "retry".parse::<Provenance>().err().map(|refusal| refusal.to_string()).unwrap_or_default();
    assert!(refusal.contains(&format!(" is no provenance: one of {leads}, agent- and a run number")), "{refusal}");
}
