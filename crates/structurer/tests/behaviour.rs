use design_model::{
    Confidence, Explicitness, Facet, NodeId, Ordinal, Prefix, SegmentRelation, Sense, SenseKind, SpanRelation,
    Specificity, Title,
};
use laws::proptest::prelude::prop_oneof;
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use newtype::Newtype;
use structurer::{Directive, Directives, Misread};
use strum::VariantArray;

struct Generated;

impl Generated {
    fn segment() -> impl Strategy<Value = structurer::SegmentRef> {
        ("[0-9a-f][0-9a-f-]{7,40}", 0usize..100_000)
            .prop_map(|(prefix, ordinal)| structurer::SegmentRef::new(Prefix::new(prefix), Ordinal::new(ordinal)))
    }

    fn node() -> impl Strategy<Value = NodeId> {
        (0..i64::MAX).prop_map(NodeId::new)
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
            (Self::segment(), Self::segment(), select(SegmentRelation::VARIANTS))
                .prop_map(|(source, target, relation)| Directive::Link { source, target, relation }),
            (Self::node(), select(Facet::VARIANTS), "[^\r\n]{0,24}").prop_map(|(refines, facet, title)| {
                Directive::Propose { refines, facet, title: Title::new(title) }
            }),
            (Self::node(), Self::node()).prop_map(|(old, by)| Directive::Supersede { old, by }),
            Self::segment().prop_map(|segment| Directive::Conversational { segment }),
        ]
    }
}

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
        "place(85b287b3.0, x1, defines)" => Spot::at(1, 19),
        "place(85b287b3.0, n1, define)" => Spot::at(1, 23),
        "place(85b287b3, n1, defines)" => Spot::at(1, 7),
        "propose(old, refines(n1), feature, \"t\")" => Spot::at(1, 9),
        "propose(new, n1, feature, \"t\")" => Spot::at(1, 14),
        "propose(new, by(n1), feature, \"t\")" => Spot::at(1, 14),
        "propose(new, refines(n1), feature, title)" => Spot::at(1, 36),
        "supersede(n1, by(n2)) extra" => Spot::at(1, 23),
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
