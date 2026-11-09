use std::io;

use design_model::{
    Basis, Confidence, Explicitness, Facet, NodeId, Ordinal, Outline, Prefix, Project, SegmentRelation, Sense,
    SenseKind, SpanRelation, Specificity, Stage, Title,
};
use dispatch_ports::Reply;
use laws::proptest::collection::vec;
use laws::proptest::option;
use laws::proptest::prelude::{Just, prop_assert_ne, prop_oneof};
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use newtype::Newtype;
use structurer::{
    Attempt, Briefing, Directive, Directives, Edge, Job, Label, Lapse, Misread, NodeRef, Proposal, Reading, Said,
    Uncovered,
};
use strum::VariantArray;
use transcript_ports::Body;

struct Generated;

impl Generated {
    fn segment() -> impl Strategy<Value = structurer::SegmentRef> {
        ("[0-9a-f][0-9a-f-]{7,40}", 0usize..100_000)
            .prop_map(|(prefix, ordinal)| structurer::SegmentRef::new(Prefix::new(prefix), Ordinal::new(ordinal)))
    }

    fn label() -> impl Strategy<Value = Label> {
        "[A-Za-z_][A-Za-z0-9_-]{0,8}".prop_filter_map("a node id is no label", |label| label.parse::<Label>().ok())
    }

    fn node() -> impl Strategy<Value = NodeRef> {
        prop_oneof![
            (0..i64::MAX).prop_map(|id| NodeRef::Stored(NodeId::new(id))),
            Self::label().prop_map(NodeRef::Labelled)
        ]
    }

    fn replacement() -> impl Strategy<Value = Edge<NodeRef>> {
        (Self::node(), Self::node())
            .prop_filter_map("a node never supersedes itself", |(old, by)| Edge::new(old, by).ok())
    }

    fn link() -> impl Strategy<Value = Edge<structurer::SegmentRef>> {
        (Self::segment(), Self::segment())
            .prop_filter_map("a segment never links to itself", |(source, target)| Edge::new(source, target).ok())
    }

    fn proposal() -> impl Strategy<Value = Proposal> {
        prop_oneof![
            Self::node().prop_map(Proposal::Anonymous),
            (Self::label(), Self::node())
                .prop_filter_map("a node never refines itself", |(label, refines)| Edge::new(label, refines).ok())
                .prop_map(Proposal::Labelled)
        ]
    }

    fn sense() -> impl Strategy<Value = Sense> {
        (
            select(SenseKind::VARIANTS),
            select(Specificity::VARIANTS),
            select(Confidence::VARIANTS),
            select(Explicitness::VARIANTS),
        )
            .prop_map(|(kind, specificity, confidence, explicitness)| {
                Sense::new(kind, specificity, confidence, explicitness)
            })
    }

    fn directive() -> impl Strategy<Value = Directive> {
        prop_oneof![
            (Self::segment(), Self::node(), select(SpanRelation::VARIANTS))
                .prop_map(|(segment, node, relation)| Directive::Place { segment, node, relation }),
            (Self::segment(), Self::sense()).prop_map(|(segment, sense)| Directive::Sense { segment, sense }),
            (Self::link(), select(SegmentRelation::VARIANTS))
                .prop_map(|(edge, relation)| Directive::Link { edge, relation }),
            (Self::proposal(), select(Facet::VARIANTS), "[^\r\n]{0,24}")
                .prop_map(|(proposal, facet, title)| Directive::Propose { proposal, facet, title: Title::new(title) }),
            Self::replacement().prop_map(Directive::Supersede),
            Self::segment().prop_map(|segment| Directive::Conversational { segment }),
        ]
    }
}

laws::roundtrip!(
    a_label_is_displayed_and_parsed_back,
    Generated::label(),
    |label: &Label| label.to_string(),
    |text: &String| text.parse::<Label>().ok(),
    Some
);

laws::roundtrip!(
    a_node_ref_is_displayed_and_parsed_back,
    Generated::node(),
    |node: &NodeRef| node.to_string(),
    |text: &String| text.parse::<NodeRef>().ok(),
    Some
);

laws::roundtrip!(
    a_directive_is_displayed_and_parsed_back,
    Generated::directive(),
    |directive: &Directive| directive.to_string(),
    |text: &String| text.parse::<Directive>().ok(),
    Some
);

laws::roundtrip!(
    directives_one_per_line_are_parsed_back_in_order,
    laws::proptest::collection::vec(Generated::directive(), 0..6),
    |directives: &Vec<Directive>| directives.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n\n"),
    |text: &String| text
        .parse::<Directives>()
        .ok()
        .map(|directives| directives.into_iter().collect::<Vec<Directive>>()),
    Some
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spot {
    line: usize,
    column: usize,
}

impl Spot {
    fn at(line: usize, column: usize) -> Option<Spot> {
        Some(line).zip(Some(column)).map(|(line, column)| Spot { line, column })
    }

    fn of_one(text: &str) -> Option<Spot> {
        Spot::of(&text.parse::<Directive>())
    }

    fn of_many(text: &str) -> Option<Spot> {
        Spot::of(&text.parse::<Directives>())
    }

    fn of<Parsed>(parsed: &Result<Parsed, Misread>) -> Option<Spot> {
        match parsed {
            Ok(_) => None,
            Err(Misread::Expected { at, .. } | Misread::Arity { at, .. }) => {
                Spot::at(at.line().into_inner(), at.column().into_inner())
            }
        }
    }
}

laws::table!(
    a_misread_directive_is_located_at_the_first_piece_out_of_place,
    Spot::of_one,
    [
        "place(85b287b3.0, n1, defines)" => None,
        "  place( 85b287b3.0 ,n1,defines )" => None,
        "plaice(85b287b3.0, n1, defines)" => Spot::at(1, 1),
        "place 85b287b3.0" => Spot::at(1, 7),
        "place(85b287b3.0, n1)" => Spot::at(1, 1),
        "place(n1, n1, defines)" => Spot::at(1, 7),
        "place(85b287b3.0, x1, defines)" => None,
        "place(85b287b3.0, n01, defines)" => Spot::at(1, 19),
        "place(85b287b3.0, n1, define)" => Spot::at(1, 23),
        "place(85b287b3, n1, defines)" => Spot::at(1, 7),
        "propose(old, refines(n1), feature, \"t\")" => Spot::at(1, 9),
        "propose(new, n1, feature, \"t\")" => Spot::at(1, 14),
        "propose(new, by(n1), feature, \"t\")" => Spot::at(1, 14),
        "propose(new, refines(n1), feature, title)" => Spot::at(1, 36),
        "supersede(n1, by(n2)) extra" => Spot::at(1, 23),
        "supersede(n1, by(n1))" => Spot::at(1, 18),
        "supersede(x, by(x))" => Spot::at(1, 17),
        "supersede(x, by(n1))" => None,
        "link(85b287b3.0, 85b287b3.0, answers)" => Spot::at(1, 18),
        "link(85b287b3.0, 85b287b3.0, refers-to)" => Spot::at(1, 18),
        "link(85b287b3.0, 85b287b3.0, supersedes)" => Spot::at(1, 18),
        "link(85b287b3.0, 85b287b3.0, corrects)" => Spot::at(1, 18),
        "link(85b287b3.0, 85b287b3.1, corrects)" => None,
        "propose(new x, refines(x), feature, \"t\")" => Spot::at(1, 24),
        "propose(new x, refines(n1), feature, \"t\")" => None,
        "propose(new  x, refines(y), feature, \"t\")" => None,
        "propose(new n2, refines(n1), feature, \"t\")" => Spot::at(1, 9),
        "propose(x, refines(n1), feature, \"t\")" => Spot::at(1, 9),
        "conversational(85b287b3.0" => Spot::at(1, 26),
        "conversational(85b287b3.0)\n" => Spot::at(1, 27),
        "conversational()" => Spot::at(1, 16),
        "sense(85b287b3.0, remark, general, firm, plain)" => Spot::at(1, 42),
        "link(85b287b3.0; 85b287b3.1, answers)" => Spot::at(1, 16),
        "propose(new, refines(n1), feature, \"open)" => Spot::at(1, 36),
        "" => Spot::at(1, 1),
    ]
);

laws::table!(
    a_misread_line_of_many_is_located_on_its_own_line,
    Spot::of_many,
    [
        "" => None,
        "\n\nconversational(85b287b3.0)\n" => None,
        "conversational(85b287b3.0)\n\n  place(85b287b3.0 n1, defines)" => Spot::at(3, 20),
        "conversational(85b287b3.0)\r\nlink(85b287b3.0, 85b287b3.1)" => Spot::at(2, 1),
        "propose(new, refines(n1), feature, \"é \\\" ü\")\nsupersede(n1)" => Spot::at(2, 1),
    ]
);

struct Jobs;

impl Jobs {
    fn said() -> impl Strategy<Value = Said> {
        (Generated::segment(), "[^\u{0}]{0,40}").prop_map(|(segment, body)| Said::new(segment, Body::new(body)))
    }

    fn node() -> impl Strategy<Value = Outline> {
        (
            (0..i64::MAX, select(Facet::VARIANTS), select(Stage::VARIANTS), select(Basis::VARIANTS)),
            (option::of("[^\u{0}]{0,24}"), option::of(0..i64::MAX), "[a-z][a-z0-9-]{0,8}"),
        )
            .prop_filter_map("a project name", |((id, facet, stage, basis), (title, parent, project))| {
                let outline =
                    Outline::new(NodeId::new(id), project.parse::<Project>().ok()?, facet, stage).based(basis);
                Some(outline.titled(title.map(Title::new)).parented(parent.map(NodeId::new)))
            })
    }

    fn any() -> impl Strategy<Value = Job> {
        (vec(Jobs::said(), 0..=Job::SEGMENTS), vec(Jobs::said(), 0..=Job::SEGMENTS), vec(Jobs::node(), 0..4))
            .prop_map(|(said, asked, nodes)| Job::new(said, asked, nodes))
    }

    fn lapse() -> impl Strategy<Value = Lapse> {
        prop_oneof![
            "[^\u{0}]{0,30}".prop_map(|refusal| Lapse::refused(&io::Error::other(refusal))),
            (Generated::segment(), vec(Generated::segment(), 0..Job::SEGMENTS))
                .prop_map(|(first, rest)| Lapse::from(Uncovered::new(first, rest))),
            "[ -~]{0,30}"
                .prop_filter_map("a misread reply", |reply| reply.parse::<Directives>().err().map(Lapse::from)),
        ]
    }

    fn attempt() -> impl Strategy<Value = Attempt> {
        prop_oneof![Just(Attempt::First), Jobs::lapse().prop_map(Attempt::Retry)]
    }
}

laws::roundtrip!(
    a_briefing_reads_back_as_it_is_rendered,
    (Jobs::any(), Jobs::attempt()),
    |drawn: &(Job, Attempt)| drawn.0.briefing(&drawn.1).to_string(),
    |text: &String| text.parse::<Briefing>().ok(),
    |drawn: (Job, Attempt)| Some(drawn.0.briefing(&drawn.1))
);

laws::law!(a_retry_briefs_the_first_briefing_and_the_lapse, (Jobs::any(), Jobs::lapse()), drawn, {
    let (job, lapse) = drawn;
    let retried = job.briefing(&Attempt::Retry(lapse.clone()));
    laws::proptest::prop_assert_eq!(&retried, &job.briefing(&Attempt::First).retried(&lapse));
    prop_assert_ne!(retried, job.briefing(&Attempt::First));
});

laws::law!(a_briefing_asks_for_no_directive_on_an_asked_line, (Jobs::any(), Jobs::attempt()), drawn, {
    let (job, attempt) = drawn;
    let rendered = job.briefing(&attempt).to_string();
    let asked_listed = rendered.contains("\nasked ");
    laws::proptest::prop_assert!(!rendered.contains("needs its own"));
    laws::proptest::prop_assert_eq!(rendered.matches("link target only").count(), 1);
    laws::proptest::prop_assert!(!asked_listed || rendered.contains("never given a directive of its own"));
});

laws::law!(a_briefing_tells_jobs_apart, (Jobs::any(), Jobs::any()), drawn, {
    let (one, other) = drawn;
    laws::proptest::prop_assume!(one != other);
    prop_assert_ne!(one.briefing(&Attempt::First), other.briefing(&Attempt::First));
});

laws::law!(a_lapse_retries_a_first_attempt_and_flails_a_retry, (Jobs::lapse(), Jobs::lapse()), drawn, {
    let (first, second) = drawn;
    let again = Attempt::First.lapsed(first.clone());
    laws::proptest::prop_assert_eq!(again, Reading::Again(Attempt::Retry(first.clone())));
    laws::proptest::prop_assert_eq!(Attempt::Retry(first).lapsed(second.clone()), Reading::Flailed(second));
});

laws::law!(
    a_reply_is_read_as_directives_or_lapses_with_its_misread,
    (Generated::directive(), Jobs::attempt()),
    drawn,
    {
        let (directive, attempt) = drawn;
        let parsed = attempt.clone().read(&Reply::new(directive.to_string()));
        let directives: Directives = directive
            .to_string()
            .parse()
            .map_err(|misread: Misread| laws::proptest::test_runner::TestCaseError::fail(misread.to_string()))?;
        laws::proptest::prop_assert_eq!(parsed, Reading::Parsed { directives, attempt: attempt.clone() });
        let misread = "place(".parse::<Directives>().err().map(Lapse::from);
        laws::proptest::prop_assert_eq!(
            Some(attempt.clone().read(&Reply::new("place(".to_owned()))),
            misread.map(|lapse| attempt.lapsed(lapse))
        );
    }
);
