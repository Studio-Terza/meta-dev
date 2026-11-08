use std::error::Error;

use design::{Failure, Line, Viewed};
use design_model::{NodeId, Segment};
use design_store::{StreamSchema, Unstructured};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::sample::select;
use laws::proptest::test_runner::TestCaseError;
use port_core::Run;
use store_ports::{Sqlite, Store};
use structurer::SegmentRef;

use super::{Applying, Invocation, Session, Spread};

const LEAD: &str = "aaaaaaaa-0000-4000-8000-000000000002.0";

const LATER: &str = "aaaaaaaa-0000-4000-8000-000000000005.0";

const OWNER: &str = "aaaaaaaa-0000-4000-8000-000000000003.0";

const OTHER: &str = "aaaaaaaa-0000-4000-8000-000000000003.1";

const DECIDED: &str = "place(aaaaaaaa-0000-4000-8000-000000000003.0, n3, decides)";

const STATED: &str = "sense(aaaaaaaa-0000-4000-8000-000000000003.0, decision, specific, firm, explicit)";

const HINTED: &str = "sense(aaaaaaaa-0000-4000-8000-000000000003.0, decision, specific, firm, implicit)";

const ASKED: &str = "place(aaaaaaaa-0000-4000-8000-000000000003.1, n3, questions)";

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
    Dropped,
    Orphaned,
    Mentioned,
    Led,
}

#[derive(Clone, Copy, Debug)]
enum Asking {
    Asked,
    Answered,
    Superseded,
    Corrected,
    Referred,
    Replaced,
    Dropped,
    Orphaned,
    Led,
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

struct Viewing;

impl Viewing {
    fn listed(view: &str, files: &[String], dropped: &[&str]) -> Option<Vec<NodeId>> {
        let applying = Applying::ready().ok()?;
        applying.applied(PLANTED).ok()?;
        files.iter().try_for_each(|file| applying.applied(file).map(drop)).ok()?;
        dropped.iter().try_for_each(|node| applying.run(&["drop", node, "--reason", "gone"]).map(drop)).ok()?;
        let lines = Spread::parsed(&applying.run(&["view", view]).ok()?);
        Some(lines.iter().filter_map(Viewing::node).collect())
    }

    fn node(line: &Line) -> Option<NodeId> {
        match line {
            Line::Viewed(Viewed::Standing { node, .. }) => Some(*node),
            _ => None,
        }
    }

    fn unplaced(line: Line) -> Option<String> {
        match line {
            Line::Viewed(Viewed::Unplaced { segment, .. }) => Some(segment.to_string()),
            _ => None,
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
            | Assent::Mentioned
            | Assent::Led => &[],
        }
    }

    fn decided(self) -> Option<Vec<NodeId>> {
        Viewing::listed("current-decision", &self.files(), self.dropped())
    }
}

laws::table!(
    a_decision_stands_while_an_owner_decides_explicitly_on_a_live_unsuperseded_node_and_nothing_supersedes_or_corrects_it,
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
            Asking::Answered => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, answers)")],
            Asking::Superseded => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, supersedes)")],
            Asking::Corrected => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, corrects)")],
            Asking::Referred => vec![ASKED.to_owned(), format!("link({OWNER}, {OTHER}, refers-to)")],
            Asking::Replaced => vec![ASKED.to_owned(), "supersede(n3, by(n4))".to_owned()],
            Asking::Led => vec![format!("place({LEAD}, n3, questions)")],
        }
    }

    fn dropped(self) -> &'static [&'static str] {
        match self {
            Asking::Dropped => &["n3"],
            Asking::Orphaned => &["n2"],
            Asking::Asked
            | Asking::Answered
            | Asking::Superseded
            | Asking::Corrected
            | Asking::Referred
            | Asking::Replaced
            | Asking::Led => &[],
        }
    }

    fn asked(self) -> Option<Vec<NodeId>> {
        Viewing::listed("open-questions", &self.files(), self.dropped())
    }
}

laws::table!(
    a_question_stays_open_on_a_live_unsuperseded_node_until_a_segment_answers_supersedes_or_corrects_it,
    |asking: &Asking| asking.asked(),
    [
        Asking::Asked => Some(vec![NodeId::new(3)]),
        Asking::Answered => Some(vec![]),
        Asking::Superseded => Some(vec![]),
        Asking::Corrected => Some(vec![]),
        Asking::Referred => Some(vec![NodeId::new(3)]),
        Asking::Replaced => Some(vec![]),
        Asking::Dropped => Some(vec![]),
        Asking::Orphaned => Some(vec![]),
        Asking::Led => Some(vec![NodeId::new(3)]),
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
    golden::golden!("view.txt", &rendered)?;
    Ok(())
}

#[test]
fn a_view_is_refused_without_a_configured_project_for_an_unknown_node_and_for_a_stray_word() -> Result<(), Failure> {
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
    Ok(())
}
