use std::error::Error;
use std::ffi::OsString;
use std::path::Path;

use design::{Cli, Count, Failure, Line, Miss, Noted, Reconciliation, Reference};
use design_model::{NodeId, NotesLine};
use design_store::StreamSchema;
use file_ports::{Bytes, Files};
use grammar::notes::Quote;
use laws::proptest::collection::vec;
use laws::proptest::prelude::{prop_assert, prop_assert_eq};
use laws::proptest::sample::select;
use laws::proptest::test_runner::TestCaseError;
use port_core::Run;
use store_ports::Sqlite;

use super::view::Viewing;
use super::{Applying, FakeLent, Invocation, Session, Speaker, Spread, Staff};

const FOUND: &str = "a1a1a1a1-0000-4000-8000-000000000001";

const PROPOSED: &str = "b2b2b2b2-0000-4000-8000-000000000002";

const APPROVED: &str = "c3c3c3c3-0000-4000-8000-000000000003";

const AGREED: &str = "d4d4d4d4-0000-4000-8000-000000000004";

const AGAIN: &str = "e5e5e5e5-0000-4000-8000-000000000005";

const REPEATED: &str = "f6f6f6f6-0000-4000-8000-000000000006";

const EXCERPT: &str = "# Garden notes (2026-11-05, draft)

Convention (owner, 2026-11-05): a line counts once it is written down.
Owner's plan: \"every bed needs a label; labels carry their rows. rows turn into a map.\" and the rest follows.
RULING (owner, explicit, 2026-11-05): \"yes the new trellis is approved.\" (new purchase)
Lead summary: one shed for every tool, as b2b2b2b2 proposed; commits 78c6b13 and ae0e9ace landed.
- 2026-11-06 RULING (owner, d4d4d4d4): 'agreed' to the planting order.
- 2026-11-07 RULING (owner): \"agreed\" again, on the compost.
- 2026-11-07 lead: the timer printed \"water again later\" and stopped.
Owner said \"agreed\" on two days.
\t
## Later, as c3c3c3c3 \"planned\"
(Owner moved on with \"onward\": not a ruling.)
";

const NOTES: &str = "notes/postmortem.md";

struct Noting;

impl Noting {
    fn session() -> Session {
        let said = [
            (
                Speaker::Owner,
                FOUND,
                "so the plan is labels. every bed needs a label; labels carry their rows. rows turn into a map. that is all",
                "2026-11-05",
            ),
            (Speaker::Lead, PROPOSED, "one shed for every tool", "2026-11-05"),
            (Speaker::Owner, APPROVED, "yes the new trellis is approved.", "2026-11-05"),
            (Speaker::Owner, AGREED, "agreed", "2026-11-06"),
            (Speaker::Owner, AGAIN, "agreed", "2026-11-07"),
            (Speaker::Owner, REPEATED, "Fine. Sow the peas. Fine.", "2026-11-08"),
        ];
        Session::working(
            said.iter().map(|(speaker, uuid, text, day)| Session::stamped(*speaker, uuid, "null", text, day)).collect(),
        )
    }

    fn ready() -> Result<Applying, Failure> {
        Applying::over(&Noting::session())
    }

    fn imported(applying: &Applying, notes: &str) -> Result<String, Failure> {
        applying.files.create_directories(Path::new("/work/notes")).ok();
        applying.files.write(&Path::new("/work").join(NOTES), Bytes::from(notes)).ok();
        applying.run(&["import-notes", NOTES])
    }

    fn outcome(line: &str) -> Option<&'static str> {
        Noting::ready()
            .ok()
            .map(|applying| Noting::imported(&applying, &format!("# Notes\n{line}\n")))
            .and_then(Noting::sorted)
    }

    fn placed(remark: &str) -> Option<Result<Vec<String>, Miss>> {
        let applying = Noting::ready().ok()?;
        let imported = Noting::imported(&applying, &format!("# Notes\nRULING (owner c3c3c3c3): {remark}\n")).ok()?;
        let shown = applying.run(&["show", "n3"]).ok()?;
        Some(Spread::parsed(&format!("{imported}{shown}")).into_iter().filter_map(Noting::found).collect())
    }

    fn found(line: Line) -> Option<Result<String, Miss>> {
        match line {
            Line::Placed { segment, .. } => Some(Ok(segment.to_string())),
            Line::Noted(Noted::Unresolved { miss, .. }) => Some(Err(miss)),
            _other => None,
        }
    }

    fn sorted(imported: Result<String, Failure>) -> Option<&'static str> {
        match imported {
            Ok(_out) => Some("imported"),
            Err(Failure::Unanchored { .. }) => Some("unanchored"),
            Err(_other) => None,
        }
    }
}

#[test]
fn import_notes_makes_a_node_of_every_content_line_anchors_each_ruling_and_view_notes_gives_the_content_back()
-> Result<(), Box<dyn Error>> {
    let applying = Noting::ready()?;
    let imported = Noting::imported(&applying, EXCERPT)?;
    let viewed = applying.run(&["view", "notes"])?;
    let shown = ["n2", "n4", "n5", "n6", "n7", "n8", "n11"]
        .iter()
        .map(|node| applying.run(&["show", node]).map(|out| format!("design show {node}\n{out}")))
        .collect::<Result<String, Failure>>()?;
    golden::golden!(
        "notes.txt",
        &format!("design import-notes {NOTES}\n{imported}design view notes\n{viewed}{shown}")
    )?;
    Ok(())
}

laws::law!(
    importing_notes_and_viewing_them_gives_the_content_back_with_as_many_nodes_as_content_lines,
    (
        vec(
            select(vec![
                ("", ""),
                ("# Notes (2026-11-05)", "# Notes (2026-11-05)"),
                ("## Later", "## Later"),
                ("Owner: \"rows turn into a map.\" and more", "Owner: \"rows turn into a map.\" and more"),
                (
                    "RULING (owner, explicit, 2026-11-05): \"yes the new trellis is approved.\"",
                    "RULING (owner, explicit, 2026-11-05): \"yes the new trellis is approved.\""
                ),
                (
                    "- 2026-11-06 RULING (owner, d4d4d4d4): 'agreed' to that",
                    "- 2026-11-06 RULING (owner, d4d4d4d4): 'agreed' to that"
                ),
                (
                    "Lead: as b2b2b2b2 said, \"water again later\" (78c6b13)",
                    "Lead: as b2b2b2b2 said, \"water again later\" (78c6b13)"
                ),
                ("(Owner moved on: not a ruling)", "(Owner moved on: not a ruling)"),
                ("  indented: prose with Rust's 'quotes' in it", "  indented: prose with Rust's 'quotes' in it"),
                ("  ", ""),
                ("\r", ""),
                ("  \r", ""),
                ("\t", ""),
                (" \t ", ""),
            ]),
            0..8
        ),
        select(vec!["\n", ""]),
    ),
    file,
    {
        let (lines, ending) = file;
        let body: String = lines.iter().flat_map(|(line, _viewed)| [*line, "\n"]).collect();
        let notes = format!("{body}last line{ending}");
        let applying = Noting::ready().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let imported =
            Noting::imported(&applying, &notes).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let viewed = applying.run(&["view", "notes"]).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let laid: String = lines.iter().flat_map(|(_line, viewed)| [*viewed, "\n"]).collect();
        let inked = lines.iter().filter(|(_line, viewed)| !viewed.is_empty()).count() + 1;
        let nodes =
            Spread::parsed(&imported).iter().filter(|line| matches!(line, Line::Noted(Noted::Imported { .. }))).count();
        prop_assert_eq!(viewed, format!("{laid}last line{ending}"));
        prop_assert_eq!(nodes, inked);
        let reconciled = format!("reconciled lines {inked} nodes {inked} ");
        prop_assert!(imported.contains(&reconciled));
    }
);

laws::table!(
    a_ruling_line_is_imported_only_when_it_quotes_owner_words_or_cites_an_owner_message,
    |line: &&str| Noting::outcome(line),
    [
        "RULING (owner): \"yes the new trellis is approved.\"" => Some("imported"),
        "RULING (owner, c3c3c3c3): approved" => Some("imported"),
        "RULING (owner): as c3c3c3c3's reply says" => Some("imported"),
        "RULING (owner): as b2b2b2b2's reply says" => Some("unanchored"),
        "RULING (owner): 'rows turn into a map.'" => Some("unanchored"),
        "RULING 'yes' (c3c3c3c3), the owners' call" => Some("imported"),
        "RULING (owner, b2b2b2b2): the lead proposed it" => Some("unanchored"),
        "RULING (owner): \"one shed for every tool\"" => Some("unanchored"),
        "RULING (owner, 2026-11-05): nothing quoted" => Some("unanchored"),
        "RULING (owner, ae0e9ace): an unknown message" => Some("unanchored"),
        "RULING (owner): \"\" says nothing" => Some("unanchored"),
        "Lead: the RULING stands" => Some("imported"),
        "(aside: RULING) prose" => Some("imported"),
        "RULING (owner, \"rows turn into a map.\"): we don't change Ann's plan" => Some("imported"),
        "RULING (owner, \"agreed\"): we don't change Ann's plan" => Some("unanchored"),
        "RULING (owner): \"rows turn into a map" => Some("unanchored"),
        "- LEAD 2026-11-08: the RULING stands" => Some("imported"),
        "* RULING (owner): nothing to anchor" => Some("unanchored"),
        "+ RULING (owner): nothing to anchor" => Some("unanchored"),
        "**RULING** (owner): nothing to anchor" => Some("unanchored"),
        "2026-11-05: RULING (owner): nothing to anchor" => Some("unanchored"),
        "- 2026-11-05: **RULING** (owner, c3c3c3c3): approved" => Some("imported"),
    ]
);

laws::table!(
    a_quote_places_only_the_segments_it_overlaps,
    |remark: &&str| Noting::placed(remark),
    [
        "\"Sow the peas. Fine\"" => Some(Ok(vec!["f6f6f6f6.1".to_owned(), "f6f6f6f6.2".to_owned()])),
        "\"Fine. Sow\"" => Some(Ok(vec!["f6f6f6f6.0".to_owned(), "f6f6f6f6.1".to_owned()])),
        "\"the peas\"" => Some(Ok(vec!["f6f6f6f6.1".to_owned()])),
        "\"the peas\", as ae0e9ace's note says" => Some(Err(Miss::Unheard)),
        "\"peas. Fine.\"" => Some(Ok(vec!["f6f6f6f6.1".to_owned(), "f6f6f6f6.2".to_owned()])),
        "\"Fine\"" => Some(Err(Miss::Ambiguous)),
        "\"Fine.\" (f6f6f6f6, 2026-11-08)" => Some(Err(Miss::Ambiguous)),
        "\"Fine. Fine\"" => Some(Err(Miss::Unheard)),
        "\"Sow the peas\" isn't Ann's call" => Some(Ok(vec!["f6f6f6f6.1".to_owned()])),
        "'Sow the peas' isn't Ann's call" => Some(Ok(vec!["c3c3c3c3.0".to_owned()])),
        "\"Sow the peas" => Some(Err(Miss::Unclosed)),
    ]
);

#[test]
fn an_imported_ruling_leaves_its_segment_to_the_structurer_or_reaches_current_decision() -> Result<(), Failure> {
    let applying = Noting::ready()?;
    Noting::imported(
        &applying,
        "# Notes\nRULING (owner, explicit): \"yes the new trellis is approved.\"\nOwner: \"rows turn into a map.\"\n- LEAD 2026-11-05: \"every bed needs a label; labels carry their rows.\"\n",
    )?;
    let decided = applying.run(&["view", "current-decision"])?;
    let deciding: Vec<NodeId> = Spread::parsed(&decided).iter().filter_map(Viewing::node).collect();
    assert_eq!(deciding, [NodeId::new(3)]);
    assert!(decided.contains("standing n3 decides c3c3c3c3.0 owner entered import-2 "));
    let unplaced: Vec<String> = Spread::parsed(&applying.run(&["view", "unplaced"])?)
        .into_iter()
        .filter_map(Viewing::unplaced)
        .filter(|segment| ["c3c3c3c3", "a1a1a1a1"].iter().any(|message| segment.contains(message)))
        .collect();
    assert_eq!(unplaced, ["a1a1a1a1.0", "a1a1a1a1.1", "a1a1a1a1.2", "a1a1a1a1.3"]);
    assert_eq!(decided.matches("a1a1a1a1").count(), 0);
    let cites =
        ["n4", "n5"].map(|node| applying.run(&["show", node]).map(|shown| shown.contains("cites a1a1a1a1 owner")));
    assert!(matches!(cites, [Ok(true), Ok(true)]));
    Ok(())
}

#[test]
fn a_ruling_anchored_only_by_an_owner_cite_is_the_current_decision() -> Result<(), Failure> {
    let applying = Noting::ready()?;
    let imported = Noting::imported(
        &applying,
        "# Notes\n- 2026-11-05 RULING (owner c3c3c3c3): we don't move Ann's trellis, it's the owner's call\nRULING (owner a1a1a1a1): labels\nRULING (owner a1a1a1a1): \"rows turn into a map.\"\n",
    )?;
    let reconciled = Reconciliation::new(Count::new(4), Count::new(4), Count::new(3), Count::new(3));
    assert!(Spread::parsed(&imported).contains(&Line::Noted(Noted::Reconciled(reconciled))));
    let decided = applying.run(&["view", "current-decision"])?;
    let deciding: Vec<NodeId> = Spread::parsed(&decided).iter().filter_map(Viewing::node).collect();
    assert_eq!(deciding, [3, 4, 4, 4, 4, 5].map(NodeId::new));
    let standing = [
        "standing n3 decides c3c3c3c3.0 owner entered import-2 ",
        "standing n4 decides a1a1a1a1.0 owner entered import-3 ",
        "standing n4 decides a1a1a1a1.1 owner entered import-3 ",
        "standing n4 decides a1a1a1a1.2 owner entered import-3 ",
        "standing n4 decides a1a1a1a1.3 owner entered import-3 ",
        "standing n5 decides a1a1a1a1.2 owner entered import-4 ",
    ];
    assert!(standing.iter().all(|part| decided.contains(part)));
    let unplaced: Vec<String> = Spread::parsed(&applying.run(&["view", "unplaced"])?)
        .into_iter()
        .filter_map(Viewing::unplaced)
        .filter(|segment| ["c3c3c3c3", "a1a1a1a1"].iter().any(|message| segment.contains(message)))
        .collect();
    assert_eq!(unplaced, Vec::<String>::new());
    Ok(())
}

#[test]
fn a_whitespace_line_is_layout_so_it_imports_as_no_node_and_views_back_empty() -> Result<(), Failure> {
    let applying = Noting::ready()?;
    let notes: String =
        ["# Notes", "  ", "first", "\t", "", " \t ", "last line"].iter().flat_map(|line| [*line, "\n"]).collect();
    let imported = Noting::imported(&applying, &notes)?;
    assert_eq!(
        Spread::parsed(&imported),
        [
            Line::Noted(Noted::Imported { line: NotesLine::new(1), node: NodeId::new(2) }),
            Line::Noted(Noted::Imported { line: NotesLine::new(3), node: NodeId::new(3) }),
            Line::Noted(Noted::Imported { line: NotesLine::new(7), node: NodeId::new(4) }),
            Line::Noted(Noted::Reconciled(Reconciliation::new(
                Count::new(3),
                Count::new(3),
                Count::new(0),
                Count::new(0)
            ))),
        ]
    );
    let laid: String =
        ["# Notes", "", "first", "", "", "", "last line"].iter().flat_map(|line| [*line, "\n"]).collect();
    assert_eq!(applying.run(&["view", "notes"])?, laid);
    Ok(())
}

#[test]
fn view_notes_gives_back_a_file_without_a_final_newline() -> Result<(), Failure> {
    let applying = Noting::ready()?;
    let notes = ["# Notes", "", "RULING (owner): as c3c3c3c3's reply says", "last line"].join("\n");
    Noting::imported(&applying, &notes)?;
    assert_eq!(applying.run(&["view", "notes"])?, notes);
    Ok(())
}

#[test]
fn an_unanchored_ruling_refuses_the_whole_import_reporting_it_and_writing_nothing() -> Result<(), Failure> {
    let applying = Noting::ready()?;
    let notes = "# Notes\nOwner: \"no such words\"\nRULING (owner): nothing to anchor\nRULING (owner): \"agreed\"\nRULING (owner, c3c3c3c3): approved\n";
    applying.files.create_directories(Path::new("/work/notes")).ok();
    applying.files.write(&Path::new("/work").join(NOTES), Bytes::from(notes)).ok();
    let mut out = Vec::new();
    let refused = Cli::new(
        &Invocation::environment(),
        FakeLent::new(&applying.files),
        |_path| applying.store.clone(),
        Staff::idle(),
    )
    .report(&Run::root(), ["import-notes", NOTES].iter().map(OsString::from), &mut out);
    let quoted = |line: usize, quote: &str, miss: Miss| {
        Line::Noted(Noted::Unresolved {
            line: NotesLine::new(line),
            reference: Reference::Quote(Quote::new(quote.to_owned())),
            miss,
        })
    };
    assert!(matches!(refused, Err(Failure::Unanchored { rulings }) if rulings == Count::new(2)));
    assert_eq!(
        Spread::parsed(&String::from_utf8(out).unwrap_or_default()),
        [
            quoted(2, "no such words", Miss::Unheard),
            Line::Noted(Noted::Unanchored(NotesLine::new(3))),
            quoted(4, "agreed", Miss::Ambiguous),
            Line::Noted(Noted::Unanchored(NotesLine::new(4))),
        ]
    );
    assert_eq!(applying.run(&["view", "notes"])?, "");
    assert!(matches!(applying.run(&["show", "n2"]), Err(Failure::Unplaced(_))));
    Ok(())
}

#[test]
fn notes_import_into_a_project_once_and_an_empty_import_holds_nothing() -> Result<(), Failure> {
    let applying = Noting::ready()?;
    let empty = [Line::Noted(Noted::Reconciled(Reconciliation::new(
        Count::new(0),
        Count::new(0),
        Count::new(0),
        Count::new(0),
    )))];
    assert_eq!(Spread::parsed(&Noting::imported(&applying, "")?), empty);
    assert_eq!(Spread::parsed(&Noting::imported(&applying, "\n \t\n\r\n")?), empty);
    assert_eq!(applying.run(&["view", "notes"])?, "");
    Noting::imported(&applying, EXCERPT)?;
    let viewed = applying.run(&["view", "notes"])?;
    assert!(matches!(Noting::imported(&applying, EXCERPT), Err(Failure::Reimported(_))));
    assert!(matches!(Noting::imported(&applying, "# Other\nsecond\n"), Err(Failure::Reimported(_))));
    assert_eq!(applying.run(&["view", "notes"])?, viewed);
    Ok(())
}

#[test]
fn notes_are_refused_without_a_project_a_readable_file_or_the_right_words() -> Result<(), Failure> {
    let store = Sqlite::in_memory::<StreamSchema>();
    Invocation::against(&store, &Noting::session(), &["ingest"])?;
    assert!(matches!(Invocation::against(&store, &Noting::session(), &["view", "notes"]), Err(Failure::Unconfigured)));
    let files = Noting::session().files();
    files.create_directories(Path::new("/work/notes")).ok();
    files.write(&Path::new("/work").join(NOTES), Bytes::from("# Notes\n")).ok();
    assert!(matches!(Invocation::within(&store, files, &["import-notes", NOTES]), Err(Failure::Unconfigured)));
    let applying = Noting::ready()?;
    assert!(matches!(applying.run(&["import-notes", "notes/absent.md"]), Err(Failure::Notes(_))));
    let strays: [&[&str]; 3] = [&["import-notes"], &["import-notes", NOTES, NOTES], &["view", "notes", "n1"]];
    let usages: Vec<&[&str]> =
        strays.into_iter().filter(|arguments| matches!(applying.run(arguments), Err(Failure::Usage))).collect();
    assert_eq!(usages, strays);
    Ok(())
}
