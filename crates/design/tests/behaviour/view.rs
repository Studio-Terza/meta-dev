use std::error::Error;

use crate::Sqlite;
use design::{Failure, Line, Unresolved, Viewed};
use design_model::{Ending, NodeId, Segment};
use design_store::{StreamSchema, Unstructured};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::sample::select;
use laws::proptest::test_runner::TestCaseError;
use newtype::Wrapping;
use port_core::Run;
use store_ports::Store;
use structurer::SegmentRef;

use super::{Applying, Invocation, Session, Spread};

const LEAD: &str = "aaaaaaaa-0000-4000-8000-000000000002.0";

const LATER: &str = "aaaaaaaa-0000-4000-8000-000000000005.0";

const OWNER: &str = "aaaaaaaa-0000-4000-8000-000000000003.0";

const OTHER: &str = "aaaaaaaa-0000-4000-8000-000000000003.1";

const DECIDED: &str = "place(aaaaaaaa-0000-4000-8000-000000000003.0, n3, decides)";

const STATED: &str = "sense(aaaaaaaa-0000-4000-8000-000000000003.0, decision, specific, firm, explicit)";

const HINTED: &str = "sense(aaaaaaaa-0000-4000-8000-000000000003.0, decision, specific, firm, implicit)";

const ASKED: &str = "place(aaaaaaaa-0000-4000-8000-000000000003.1, n3, questions)\nsense(aaaaaaaa-0000-4000-8000-000000000003.1, question, specific, firm, explicit)";

const PLACED: &str = "place(aaaaaaaa-0000-4000-8000-000000000003.1, n3, questions)";

const BEGUN: &str = "place(aaaaaaaa-0000-4000-8000-000000000001.0, n3, questions)\nsense(aaaaaaaa-0000-4000-8000-000000000001.0, question, specific, firm, explicit)";

const CHILD: &str = "place(aaaaaaaa-0000-4000-8000-000000000003.0, n3, decides)";

const PLANTED: &str = "propose(new, refines(n1), feature, \"parent\")\npropose(new, refines(n2), feature, \"placed\")\npropose(new, refines(n1), feature, \"successor\")";

#[derive(Clone, Copy, Debug)]
enum Assent {
    Stated,
    Hinted,
    Unsensed,
    Revised,
    Reaffirmed,
    Superseded,
    Corrected,
    Replaced,
    Inherited,
    Ancestral,
    Dropped,
    Orphaned,
    Mentioned,
    Led,
}

#[derive(Clone, Copy, Debug)]
enum Asking {
    Asked,
    Unsensed,
    Requested,
    Rhetorical,
    Resensed,
    Reasked,
    Answered,
    LeadAnswered,
    DecidedInTurn,
    DecidedElsewhere,
    DecidedLater,
    HintedInTurn,
    Superseded,
    Corrected,
    Referred,
    Replaced,
    Inherited,
    Ancestral,
    Dropped,
    Orphaned,
    Led,
}

#[derive(Clone, Copy, Debug)]
enum Derivation {
    Bare,
    Own,
    Child,
    Hinted,
    Revised,
    Sibling,
    Questioned,
    Led,
    Specified,
    Refined,
}

#[derive(Clone, Copy, Debug)]
enum Own {
    Bare,
    Placed,
    Sensed,
    Conversational,
    Linking,
    Linked,
}

#[derive(Clone, Copy, Debug)]
enum Owed {
    Unplaced,
    Covered,
}

pub(super) struct Viewing;

impl Viewing {
    fn listed(view: &str, files: &[String], dropped: &[&str]) -> Option<Vec<NodeId>> {
        let applying = Applying::ready().ok()?;
        applying.applied(PLANTED).ok()?;
        files
            .iter()
            .try_for_each(|file| {
                applying.applied(file).map(drop).or_else(|failure| {
                    matches!(failure, Failure::Unresolved(Unresolved::Ancestral { .. })).then_some(()).ok_or(failure)
                })
            })
            .ok()?;
        dropped.iter().try_for_each(|node| applying.run(&["drop", node, "--reason", "gone"]).map(drop)).ok()?;
        let lines = Spread::parsed(&applying.run(&["view", view]).ok()?);
        Some(lines.iter().filter_map(Viewing::node).collect())
    }

    pub(super) fn node(line: &Line) -> Option<NodeId> {
        match line {
            Line::Viewed(Viewed::Standing { node, .. }) => Some(*node),
            Line::Viewed(Viewed::Outlined(_) | Viewed::Unplaced { .. })
            | Line::Read { .. }
            | Line::Authored { .. }
            | Line::Unrecognised { .. }
            | Line::Formed { .. }
            | Line::Related { .. }
            | Line::Written(_)
            | Line::Hit { .. }
            | Line::Root { .. }
            | Line::Consumes { .. }
            | Line::Node(_)
            | Line::Artifact(_)
            | Line::Condition(_)
            | Line::Reason(_)
            | Line::Relative { .. }
            | Line::Edge { .. }
            | Line::Role { .. }
            | Line::Placed { .. }
            | Line::Message { .. }
            | Line::Segment { .. }
            | Line::Link { .. }
            | Line::Refused(_)
            | Line::Annotated { .. }
            | Line::Applied(_)
            | Line::Proposed { .. }
            | Line::Uncovered(_)
            | Line::Repeated { .. }
            | Line::Reaped { .. }
            | Line::Interrupted(_)
            | Line::Ran(_)
            | Line::Retried(_)
            | Line::Flailed(_)
            | Line::FlailedSegment(_)
            | Line::Owed(_)
            | Line::Noted(_) => None,
        }
    }

    fn outlined(line: Line) -> Option<String> {
        match line {
            Line::Viewed(Viewed::Outlined(outline)) => {
                Some(format!("n{} {} {}", outline.node().into_inner(), outline.stage(), outline.basis()))
            }
            Line::Viewed(Viewed::Standing { .. } | Viewed::Unplaced { .. })
            | Line::Read { .. }
            | Line::Authored { .. }
            | Line::Unrecognised { .. }
            | Line::Formed { .. }
            | Line::Related { .. }
            | Line::Written(_)
            | Line::Hit { .. }
            | Line::Root { .. }
            | Line::Consumes { .. }
            | Line::Node(_)
            | Line::Artifact(_)
            | Line::Condition(_)
            | Line::Reason(_)
            | Line::Relative { .. }
            | Line::Edge { .. }
            | Line::Role { .. }
            | Line::Placed { .. }
            | Line::Message { .. }
            | Line::Segment { .. }
            | Line::Link { .. }
            | Line::Refused(_)
            | Line::Annotated { .. }
            | Line::Applied(_)
            | Line::Proposed { .. }
            | Line::Uncovered(_)
            | Line::Repeated { .. }
            | Line::Reaped { .. }
            | Line::Interrupted(_)
            | Line::Ran(_)
            | Line::Retried(_)
            | Line::Flailed(_)
            | Line::FlailedSegment(_)
            | Line::Owed(_)
            | Line::Noted(_) => None,
        }
    }

    pub(super) fn unplaced(line: Line) -> Option<String> {
        match line {
            Line::Viewed(Viewed::Unplaced { segment, .. }) => Some(segment.to_string()),
            Line::Viewed(Viewed::Outlined(_) | Viewed::Standing { .. })
            | Line::Read { .. }
            | Line::Authored { .. }
            | Line::Unrecognised { .. }
            | Line::Formed { .. }
            | Line::Related { .. }
            | Line::Written(_)
            | Line::Hit { .. }
            | Line::Root { .. }
            | Line::Consumes { .. }
            | Line::Node(_)
            | Line::Artifact(_)
            | Line::Condition(_)
            | Line::Reason(_)
            | Line::Relative { .. }
            | Line::Edge { .. }
            | Line::Role { .. }
            | Line::Placed { .. }
            | Line::Message { .. }
            | Line::Segment { .. }
            | Line::Link { .. }
            | Line::Refused(_)
            | Line::Annotated { .. }
            | Line::Applied(_)
            | Line::Proposed { .. }
            | Line::Uncovered(_)
            | Line::Repeated { .. }
            | Line::Reaped { .. }
            | Line::Interrupted(_)
            | Line::Ran(_)
            | Line::Retried(_)
            | Line::Flailed(_)
            | Line::FlailedSegment(_)
            | Line::Owed(_)
            | Line::Noted(_) => None,
        }
    }
}

impl Assent {
    fn files(self) -> Vec<String> {
        match self {
            Assent::Stated | Assent::Dropped | Assent::Orphaned => vec![format!("{DECIDED}\n{STATED}")],
            Assent::Hinted => vec![format!("{DECIDED}\n{HINTED}")],
            Assent::Unsensed => vec![DECIDED.to_owned()],
            Assent::Revised => vec![format!("{DECIDED}\n{STATED}"), HINTED.to_owned()],
            Assent::Reaffirmed => vec![format!("{DECIDED}\n{HINTED}"), STATED.to_owned()],
            Assent::Superseded => vec![format!("{DECIDED}\n{STATED}"), format!("link({OTHER}, {OWNER}, supersedes)")],
            Assent::Corrected => vec![format!("{DECIDED}\n{STATED}"), format!("link({OTHER}, {OWNER}, corrects)")],
            Assent::Replaced => vec![format!("{DECIDED}\n{STATED}"), "supersede(n3, by(n4))".to_owned()],
            Assent::Inherited => vec![format!("{DECIDED}\n{STATED}"), "supersede(n2, by(n4))".to_owned()],
            Assent::Ancestral => vec![format!("{DECIDED}\n{STATED}"), "supersede(n2, by(n3))".to_owned()],
            Assent::Mentioned => vec![format!("place({OWNER}, n3, mentions)\n{STATED}")],
            Assent::Led => {
                vec![format!("place({LEAD}, n3, decides)\nsense({LEAD}, decision, specific, firm, explicit)")]
            }
        }
    }

    fn dropped(self) -> &'static [&'static str] {
        match self {
            Assent::Dropped => &["n3"],
            Assent::Orphaned => &["n2"],
            Assent::Stated
            | Assent::Hinted
            | Assent::Unsensed
            | Assent::Revised
            | Assent::Reaffirmed
            | Assent::Superseded
            | Assent::Corrected
            | Assent::Replaced
            | Assent::Inherited
            | Assent::Ancestral
            | Assent::Mentioned
            | Assent::Led => &[],
        }
    }

    fn decided(self) -> Option<Vec<NodeId>> {
        Viewing::listed("current-decision", &self.files(), self.dropped())
    }
}

laws::table!(
    a_decision_stands_while_an_owner_decides_explicitly_on_a_current_node_and_nothing_supersedes_or_corrects_it_and_a_decision_under_a_superseded_node_falls_with_it,
    |assent: &Assent| assent.decided(),
    [
        Assent::Stated => Some(vec![NodeId::new(3)]),
        Assent::Hinted => Some(vec![]),
        Assent::Unsensed => Some(vec![]),
        Assent::Revised => Some(vec![]),
        Assent::Reaffirmed => Some(vec![NodeId::new(3)]),
        Assent::Superseded => Some(vec![]),
        Assent::Corrected => Some(vec![]),
        Assent::Replaced => Some(vec![]),
        Assent::Inherited => Some(vec![]),
        Assent::Ancestral => Some(vec![NodeId::new(3)]),
        Assent::Dropped => Some(vec![]),
        Assent::Orphaned => Some(vec![]),
        Assent::Mentioned => Some(vec![]),
        Assent::Led => Some(vec![]),
    ]
);

impl Asking {
    fn files(self) -> Vec<String> {
        match self {
            Asking::Asked | Asking::Dropped | Asking::Orphaned => vec![ASKED.to_owned()],
            Asking::Unsensed => vec![PLACED.to_owned()],
            Asking::Requested => vec![format!("{PLACED}\nsense({OTHER}, ask, specific, firm, explicit)")],
            Asking::Rhetorical => vec![format!("{PLACED}\nsense({OTHER}, opinion, specific, firm, explicit)")],
            Asking::Resensed => vec![ASKED.to_owned(), format!("sense({OTHER}, ask, specific, firm, explicit)")],
            Asking::Reasked => vec![
                format!("{PLACED}\nsense({OTHER}, ask, specific, firm, explicit)"),
                format!("sense({OTHER}, question, specific, firm, explicit)"),
            ],
            Asking::Answered => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, answers)")],
            Asking::LeadAnswered => vec![ASKED.to_owned(), format!("link({LEAD}, {OTHER}, answers)")],
            Asking::DecidedInTurn => vec![BEGUN.to_owned(), format!("{DECIDED}\n{STATED}")],
            Asking::DecidedElsewhere => {
                vec![BEGUN.to_owned(), format!("place({OWNER}, n2, decides)\n{STATED}")]
            }
            Asking::DecidedLater => vec![
                BEGUN.to_owned(),
                format!("place({LATER}, n3, decides)\nsense({LATER}, decision, specific, firm, explicit)"),
            ],
            Asking::HintedInTurn => vec![BEGUN.to_owned(), format!("{DECIDED}\n{HINTED}")],
            Asking::Superseded => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, supersedes)")],
            Asking::Corrected => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, corrects)")],
            Asking::Referred => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, refers-to)")],
            Asking::Replaced => vec![ASKED.to_owned(), "supersede(n3, by(n4))".to_owned()],
            Asking::Inherited => vec![ASKED.to_owned(), "supersede(n2, by(n4))".to_owned()],
            Asking::Ancestral => vec![ASKED.to_owned(), "supersede(n2, by(n3))".to_owned()],
            Asking::Led => {
                vec![format!("place({LEAD}, n3, questions)\nsense({LEAD}, question, specific, firm, explicit)")]
            }
        }
    }

    fn dropped(self) -> &'static [&'static str] {
        match self {
            Asking::Dropped => &["n3"],
            Asking::Orphaned => &["n2"],
            Asking::Asked
            | Asking::Unsensed
            | Asking::Requested
            | Asking::Rhetorical
            | Asking::Resensed
            | Asking::Reasked
            | Asking::Answered
            | Asking::LeadAnswered
            | Asking::DecidedInTurn
            | Asking::DecidedElsewhere
            | Asking::DecidedLater
            | Asking::HintedInTurn
            | Asking::Superseded
            | Asking::Corrected
            | Asking::Referred
            | Asking::Replaced
            | Asking::Inherited
            | Asking::Ancestral
            | Asking::Led => &[],
        }
    }

    fn asked(self) -> Option<Vec<NodeId>> {
        Viewing::listed("open-questions", &self.files(), self.dropped())
    }
}

laws::table!(
    a_question_sensed_as_one_stays_open_on_a_current_node_until_an_owner_segment_answers_supersedes_or_corrects_it_or_the_owner_s_next_message_decides_its_node,
    |asking: &Asking| asking.asked(),
    [
        Asking::Asked => Some(vec![NodeId::new(3)]),
        Asking::Unsensed => Some(vec![]),
        Asking::Requested => Some(vec![]),
        Asking::Rhetorical => Some(vec![]),
        Asking::Resensed => Some(vec![]),
        Asking::Reasked => Some(vec![NodeId::new(3)]),
        Asking::Answered => Some(vec![]),
        Asking::LeadAnswered => Some(vec![NodeId::new(3)]),
        Asking::DecidedInTurn => Some(vec![]),
        Asking::DecidedElsewhere => Some(vec![NodeId::new(3)]),
        Asking::DecidedLater => Some(vec![NodeId::new(3)]),
        Asking::HintedInTurn => Some(vec![NodeId::new(3)]),
        Asking::Superseded => Some(vec![]),
        Asking::Corrected => Some(vec![]),
        Asking::Referred => Some(vec![NodeId::new(3)]),
        Asking::Replaced => Some(vec![]),
        Asking::Inherited => Some(vec![]),
        Asking::Ancestral => Some(vec![NodeId::new(3)]),
        Asking::Dropped => Some(vec![]),
        Asking::Orphaned => Some(vec![]),
        Asking::Led => Some(vec![NodeId::new(3)]),
    ]
);

impl Derivation {
    fn files(self) -> Vec<String> {
        match self {
            Derivation::Bare | Derivation::Refined => Vec::new(),
            Derivation::Own => vec![format!("place({OWNER}, n2, decides)\n{STATED}")],
            Derivation::Child | Derivation::Specified => vec![format!("{CHILD}\n{STATED}")],
            Derivation::Hinted => vec![format!("{CHILD}\n{HINTED}")],
            Derivation::Revised => vec![format!("{CHILD}\n{STATED}"), HINTED.to_owned()],
            Derivation::Sibling => vec![format!("place({OWNER}, n4, decides)\n{STATED}")],
            Derivation::Questioned => vec![ASKED.to_owned()],
            Derivation::Led => {
                vec![format!("place({LEAD}, n3, decides)\nsense({LEAD}, decision, specific, firm, explicit)")]
            }
        }
    }

    fn moves(self) -> &'static [&'static [&'static str]] {
        match self {
            Derivation::Specified => {
                &[&["refine", "n2"], &["specify", "n2", "--artifact", "behaviour:p", "--condition", "p holds"]]
            }
            Derivation::Refined => &[&["refine", "n2"]],
            Derivation::Bare
            | Derivation::Own
            | Derivation::Child
            | Derivation::Hinted
            | Derivation::Revised
            | Derivation::Sibling
            | Derivation::Questioned
            | Derivation::Led => &[],
        }
    }

    fn staged(self) -> Option<Vec<String>> {
        let applying = Applying::ready().ok()?;
        applying.applied(PLANTED).ok()?;
        self.moves().iter().try_for_each(|arguments| applying.run(arguments).map(drop)).ok()?;
        self.files().iter().try_for_each(|file| applying.applied(file).map(drop)).ok()?;
        let lines = Spread::parsed(&applying.run(&["view", "outline"]).ok()?);
        Some(lines.into_iter().filter_map(Viewing::outlined).collect())
    }
}

laws::table!(
    a_node_refines_while_an_owner_decision_stands_on_it_or_beneath_it_unless_the_lead_staged_it,
    |derivation: &Derivation| derivation.staged(),
    [
        Derivation::Bare => Some(vec![
            "n1 vague vague".to_owned(),
            "n2 vague vague".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Own => Some(vec![
            "n1 refining decided".to_owned(),
            "n2 refining decided".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Child => Some(vec![
            "n1 refining decided".to_owned(),
            "n2 refining decided".to_owned(),
            "n3 refining decided".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Hinted => Some(vec![
            "n1 vague vague".to_owned(),
            "n2 vague vague".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Revised => Some(vec![
            "n1 vague vague".to_owned(),
            "n2 vague vague".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Sibling => Some(vec![
            "n1 refining decided".to_owned(),
            "n2 vague vague".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 refining decided".to_owned(),
        ]),
        Derivation::Questioned => Some(vec![
            "n1 vague vague".to_owned(),
            "n2 vague vague".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Led => Some(vec![
            "n1 vague vague".to_owned(),
            "n2 vague vague".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Specified => Some(vec![
            "n1 refining decided".to_owned(),
            "n2 terminal set".to_owned(),
            "n3 refining decided".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
        Derivation::Refined => Some(vec![
            "n1 vague vague".to_owned(),
            "n2 refining set".to_owned(),
            "n3 vague vague".to_owned(),
            "n4 vague vague".to_owned(),
        ]),
    ]
);

impl Own {
    const ALL: [Own; 6] = [Own::Bare, Own::Placed, Own::Sensed, Own::Conversational, Own::Linking, Own::Linked];

    fn directive(self, segment: &str) -> Option<String> {
        match self {
            Own::Bare => None,
            Own::Placed => Some(format!("place({segment}, n1, mentions)")),
            Own::Sensed => Some(format!("sense({segment}, remark, general, tentative, implicit)")),
            Own::Conversational => Some(format!("conversational({segment})")),
            Own::Linking => Some(format!("link({segment}, {LEAD}, refers-to)")),
            Own::Linked => Some(format!("link({LEAD}, {segment}, refers-to)")),
        }
    }

    fn owed(self) -> Owed {
        match self {
            Own::Bare | Own::Linked => Owed::Unplaced,
            Own::Placed | Own::Sensed | Own::Conversational | Own::Linking => Owed::Covered,
        }
    }
}

laws::law!(
    the_unplaced_view_lists_exactly_the_owner_segments_without_a_directive_of_their_own,
    (select(Own::ALL.to_vec()), select(Own::ALL.to_vec()), select(Own::ALL.to_vec()), select(Own::ALL.to_vec())),
    owns,
    {
        let segments = [format!("{}.0", Session::ASKED), Applying::answer(0), Applying::answer(1), LATER.to_owned()];
        let owned = segments.iter().zip([owns.0, owns.1, owns.2, owns.3]);
        let directives: Vec<String> = owned.clone().filter_map(|(segment, own)| own.directive(segment)).collect();
        let applying = Applying::ready().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        applying
            .applied(&["propose(new, refines(n1), feature, \"cover\")".to_owned(), directives.join("\n")].join("\n"))
            .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let viewed = applying.run(&["view", "unplaced"]).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let listed: Vec<String> = Spread::parsed(&viewed).into_iter().filter_map(Viewing::unplaced).collect();
        let wanted: Vec<String> = owned
            .filter(|(_segment, own)| matches!(own.owed(), Owed::Unplaced))
            .map(|(segment, _own)| segment.clone())
            .collect();
        let owed = Run::root()
            .port(applying.store)
            .read(Unstructured)
            .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let unstructured: Vec<String> =
            owed.iter().map(|unplaced| SegmentRef::from(AsRef::<Segment>::as_ref(unplaced)).to_string()).collect();
        prop_assert_eq!(&listed, &wanted);
        prop_assert_eq!(unstructured, wanted);
    }
);

#[test]
fn each_view_renders_the_live_tree_its_standing_decisions_open_questions_and_unplaced_words()
-> Result<(), Box<dyn Error>> {
    let applying = Applying::ready()?;
    applying.applied(
        &[
            "propose(new kept, refines(n1), feature, \"kept\")",
            "propose(new detail, refines(kept), feature, \"detail\")",
            "propose(new older, refines(n1), feature, \"older\")",
            "propose(new newer, refines(n1), feature, \"newer\")",
            "supersede(older, by(newer))",
            "propose(new spare, refines(n1), smell, \"spare\")",
            "propose(new under, refines(spare), feature, \"under\")",
        ]
        .join("\n"),
    )?;
    let [decided, asked] = [Applying::answer(0), Applying::answer(1)];
    applying.applied(
        &[
            format!("place({decided}, n2, decides)"),
            format!("place({decided}, n4, decides)"),
            format!("place({decided}, n7, decides)"),
            format!("sense({decided}, decision, specific, firm, explicit)"),
            format!("place({asked}, n3, questions)"),
            format!("place({asked}, n5, questions)"),
            format!("sense({asked}, question, specific, firm, explicit)"),
        ]
        .join("\n"),
    )?;
    applying.run(&["drop", "n6", "--reason", "folded into n2"])?;
    let views: [&[&str]; 5] = [
        &["view", "outline"],
        &["view", "outline", "n2"],
        &["view", "current-decision"],
        &["view", "open-questions"],
        &["view", "unplaced"],
    ];
    let rendered = views
        .iter()
        .map(|arguments| applying.run(arguments).map(|out| format!("design {}\n{out}", arguments.join(" "))))
        .collect::<Result<String, Failure>>()?;
    golden::golden!("view.txt", &rendered);
    Ok(())
}

#[test]
fn a_view_rooted_at_a_dropped_node_is_refused_as_is_one_at_a_superseded_node_without_a_configured_project_at_an_unknown_node_or_with_a_stray_word()
-> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    Invocation::against(&store, &Session::linked(), &["ingest"])?;
    assert!(matches!(
        Invocation::against(&store, &Session::linked(), &["view", "outline"]),
        Err(Failure::Unconfigured)
    ));
    let applying = Applying::ready()?;
    assert!(matches!(applying.run(&["view", "open-questions", "n9"]), Err(Failure::Unplaced(_))));
    let strays: [&[&str]; 5] = [
        &["view"],
        &["view", "unplaced", "n1"],
        &["view", "outlines"],
        &["view", "outline", "n1", "n1"],
        &["view", "outline", "--all"],
    ];
    let usages: Vec<&[&str]> =
        strays.into_iter().filter(|arguments| matches!(applying.run(arguments), Err(Failure::Usage))).collect();
    assert_eq!(usages, strays);
    applying.applied(&format!("{PLANTED}\npropose(new, refines(n1), feature, \"replaced\")\nsupersede(n5, by(n4))"))?;
    applying.run(&["drop", "n2", "--reason", "gone"])?;
    let ended: [(&[&str], NodeId, Ending); 3] = [
        (&["view", "outline", "n2"], NodeId::new(2), Ending::Dropped),
        (&["view", "current-decision", "n3"], NodeId::new(3), Ending::Dropped),
        (&["view", "open-questions", "n5"], NodeId::new(5), Ending::Superseded),
    ];
    let refused: Vec<(&[&str], NodeId, Ending)> = ended
        .into_iter()
        .filter(|(arguments, node, ending)| {
            matches!(applying.run(arguments), Err(Failure::Ended { node: refused, ending: why }) if refused == *node && why == *ending)
        })
        .collect();
    assert_eq!(refused, ended);
    Ok(())
}
