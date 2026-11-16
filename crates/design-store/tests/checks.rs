use design_jobs::JobState;
use design_model::{
    Confidence, Explicitness, Facet, NodeRelation, Ordinal, Role, SegmentRelation, SenseKind, SpanRelation,
    Specificity, Stage,
};
use design_runs::{Delivery, Halted, RunState};
use design_store::StreamSchema;
use dispatch_ports::{Pid, Tokens};
use sqlite_ports::SqliteOpener;

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

laws::checked!(
    every_listed_and_counted_column_refuses_what_its_type_refuses,
    Sqlite::in_memory::<StreamSchema>(),
    [
        "nodes"."facet": Facet,
        "nodes"."stage": Stage,
        "nodes"."role": Role,
        "node_edges"."relation": NodeRelation,
        "span_edges"."relation": SpanRelation,
        "span_edges"."ordinal": Ordinal,
        "segment_links"."relation": SegmentRelation,
        "segment_links"."source_ordinal": Ordinal,
        "segment_links"."target_ordinal": Ordinal,
        "senses"."kind": SenseKind,
        "senses"."specificity": Specificity,
        "senses"."confidence": Confidence,
        "senses"."explicitness": Explicitness,
        "senses"."ordinal": Ordinal,
        "segments"."ordinal": Ordinal,
        "conversational"."ordinal": Ordinal,
        "flailed"."ordinal": Ordinal,
        "jobs"."state": JobState,
        "runs"."state": RunState,
        "runs"."pid": Pid,
        "runs"."input_tokens": Option<Tokens>,
        "runs"."output_tokens": Option<Tokens>,
        "runs"."cache_written_tokens": Option<Tokens>,
        "runs"."cache_read_tokens": Option<Tokens>,
        "failures"."halted": Halted,
        "failures"."delivery": Delivery,
    ]
);
