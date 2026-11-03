use lint_port_batching::PortBatching;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

macro_rules! ported {
    ($body:literal) => {
        concat!(
            "mod ports {\n    pub trait Store {\n        fn fetch(&self, id: u8) -> u8;\n        fn paced(&self, id: u8, pacing: Pacing) -> u8;\n    }\n}\nuse ports::Store;\n",
            $body
        )
    };
}

const MAPPED: &str = ported!(
    "fn f(store: &impl Store, ids: Vec<u8>) -> Vec<u8> {\n    ids.iter().map(|id| store.fetch(*id)).collect()\n}\n"
);
const LOOPED: &str =
    ported!("fn f(store: &dyn Store, ids: Vec<u8>) {\n    for id in ids {\n        store.fetch(id);\n    }\n}\n");
const PASSED_ON: &str =
    ported!("fn f<S: Store>(store: &S, ids: Vec<u8>) {\n    ids.into_iter().for_each(|id| g(store, id));\n}\n");
const WHILE: &str = ported!(
    "fn f<S>(store: &S, mut n: u8)\nwhere\n    S: Store,\n{\n    while n > 0 {\n        n = store.fetch(n);\n    }\n}\n"
);
const HELD: &str = ported!(
    "struct Shelf<'a, S: Store> {\n    store: &'a S,\n}\nimpl<'a, S: Store> Shelf<'a, S> {\n    fn all(&self, ids: &[u8]) -> Vec<u8> {\n        ids.iter().map(|id| self.store.fetch(*id)).collect()\n    }\n}\n"
);
const HOLDER: &str = ported!(
    "struct Shelf<'a> {\n    store: &'a dyn Store,\n}\nfn f(shelf: &Shelf<'_>, ids: Vec<u8>) {\n    ids.iter().for_each(|id| shelf.get(*id));\n}\n"
);
const RANGE: &str =
    ported!("fn f(store: &impl Store) -> Vec<u8> {\n    (0..3).map(|id| store.fetch(id)).collect()\n}\n");
const STREAM: &str =
    ported!("fn f(store: &impl Store, out: &mut Vec<u8>) {\n    store.stream().for_each(|item| out.push(item));\n}\n");
const STREAM_LOOP: &str =
    ported!("fn f(store: &impl Store) {\n    for item in store.stream() {\n        drop(item);\n    }\n}\n");
const BATCHED: &str = ported!(
    "fn f(store: &impl Store, ids: Vec<u8>) -> Vec<u8> {\n    let wanted: Vec<u8> = ids.iter().map(|id| id + 1).collect();\n    store.fetch_many(&wanted)\n}\n"
);
const OPTION: &str =
    ported!("fn f(store: &impl Store, id: Option<u8>) -> Option<u8> {\n    id.map(|id| store.fetch(id))\n}\n");
const FOUND: &str = ported!(
    "fn f(store: &impl Store, ids: Vec<u8>) -> Option<u8> {\n    ids.iter().find(|id| **id > 1).map(|id| store.fetch(*id))\n}\n"
);
const PACED: &str = ported!(
    "fn f(store: &impl Store, ids: Vec<u8>) {\n    ids.iter().for_each(|id| {\n        store.paced(*id, Pacing::RateLimit);\n    });\n}\n"
);
const OTHER: &str = ported!(
    "trait Other {\n    fn fetch(&self, id: u8) -> u8;\n}\nfn f(other: &impl Other, ids: Vec<u8>) -> Vec<u8> {\n    ids.iter().map(|id| other.fetch(*id)).collect()\n}\n"
);
const SHADOWED: &str = ported!(
    "fn f(store: &impl Store, stores: Vec<u8>) -> Vec<u8> {\n    stores.iter().map(|store: &u8| store + 1).collect()\n}\n"
);
const TEST_MODULE: &str = ported!(
    "#[cfg(test)]\nmod tests {\n    fn f(store: &impl super::Store, ids: Vec<u8>) {\n        for id in ids {\n            store.fetch(id);\n        }\n    }\n}\n"
);
const PLAIN_FIELD: &str = ported!(
    "struct Shelf<'a> {\n    store: &'a dyn Store,\n}\nstruct Plain {\n    store: u8,\n}\nimpl Plain {\n    fn all(&self, ids: &[u8]) -> Vec<u8> {\n        ids.iter().map(|id| self.store + id).collect()\n    }\n}\nfn f(plain: &Plain, ids: Vec<u8>) {\n    ids.iter().for_each(|id| plain.all(&[*id]));\n}\n"
);
const INLINE_MODULE: &str = ported!(
    "mod inner {\n    use crate::ports::Store;\n    struct Shelf<'a> {\n        store: &'a dyn Store,\n    }\n    impl Shelf<'_> {\n        fn all(&self, ids: &[u8]) {\n            for id in ids {\n                self.store.fetch(*id);\n            }\n        }\n    }\n}\n"
);
const DEFAULT_METHOD: &str = ported!(
    "trait Shelf {\n    fn all(store: &impl Store, ids: Vec<u8>) -> Vec<u8> {\n        ids.iter().map(|id| store.fetch(*id)).collect()\n    }\n}\n"
);
const GENERATED: &str = ported!(
    "macro_rules! shelf {\n    () => {\n        fn f(store: &impl Store, ids: Vec<u8>) -> Vec<u8> {\n            ids.iter().map(|id| store.fetch(*id)).collect()\n        }\n    };\n}\n"
);
const PACED_FIELD: &str = ported!(
    "struct Shelf<'a> {\n    store: &'a dyn Store,\n}\nimpl Shelf<'_> {\n    fn all(&self, ids: &[u8]) {\n        ids.iter().for_each(|id| {\n            self.store.paced(*id, Pacing::RateLimit);\n        });\n    }\n}\n"
);
const BARE_LOOP: &str = ported!("fn f(store: &impl Store) {\n    loop {\n        store.fetch(1);\n    }\n}\n");
const IMPL_GENERIC: &str = ported!(
    "struct Shelf;\nimpl<S: Store> Shelf {\n    fn all(store: &S, ids: Vec<u8>) -> Vec<u8> {\n        ids.iter().map(|id| store.fetch(*id)).collect()\n    }\n}\n"
);
const BOXED: &str = ported!(
    "fn f(store: Box<dyn Store>, ids: Vec<u8>) -> Vec<u8> {\n    ids.iter().map(|id| store.fetch(*id)).collect()\n}\n"
);

const CASES: [Case; 24] = [
    Case::new(
        Title::new("closure passed to map over an iterator"),
        &[Source::new(Located::new("a.rs"), Text::new(MAPPED))],
        Flagged::new(&["a.rs:9"]),
    ),
    Case::new(
        Title::new("for loop body"),
        &[Source::new(Located::new("a.rs"), Text::new(LOOPED))],
        Flagged::new(&["a.rs:10"]),
    ),
    Case::new(
        Title::new("port passed on inside for_each"),
        &[Source::new(Located::new("a.rs"), Text::new(PASSED_ON))],
        Flagged::new(&["a.rs:9"]),
    ),
    Case::new(
        Title::new("where clause and while loop"),
        &[Source::new(Located::new("a.rs"), Text::new(WHILE))],
        Flagged::new(&["a.rs:13"]),
    ),
    Case::new(
        Title::new("field holding a port"),
        &[Source::new(Located::new("a.rs"), Text::new(HELD))],
        Flagged::new(&["a.rs:13"]),
    ),
    Case::new(
        Title::new("holder passed as a parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(HOLDER))],
        Flagged::new(&["a.rs:12"]),
    ),
    Case::new(Title::new("range"), &[Source::new(Located::new("a.rs"), Text::new(RANGE))], Flagged::new(&["a.rs:9"])),
    Case::new(
        Title::new("stream consumed from a port"),
        &[Source::new(Located::new("a.rs"), Text::new(STREAM))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("loop over a stream from a port"),
        &[Source::new(Located::new("a.rs"), Text::new(STREAM_LOOP))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("batch call outside the closure"),
        &[Source::new(Located::new("a.rs"), Text::new(BATCHED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("map on an option"),
        &[Source::new(Located::new("a.rs"), Text::new(OPTION))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("map after find yields an option"),
        &[Source::new(Located::new("a.rs"), Text::new(FOUND))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("paced port method"),
        &[Source::new(Located::new("a.rs"), Text::new(PACED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait outside ports"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("shadowed by a closure parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOWED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("plain field named like a held one"),
        &[Source::new(Located::new("a.rs"), Text::new(PLAIN_FIELD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_MODULE))],
        Flagged::new(&["a.rs:16"]),
    ),
    Case::new(
        Title::new("trait default method"),
        &[Source::new(Located::new("a.rs"), Text::new(DEFAULT_METHOD))],
        Flagged::new(&["a.rs:10"]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERATED))],
        Flagged::new(&["a.rs:11"]),
    ),
    Case::new(
        Title::new("paced through a held field"),
        &[Source::new(Located::new("a.rs"), Text::new(PACED_FIELD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("bare loop"),
        &[Source::new(Located::new("a.rs"), Text::new(BARE_LOOP))],
        Flagged::new(&["a.rs:10"]),
    ),
    Case::new(
        Title::new("port generic declared on the impl"),
        &[Source::new(Located::new("a.rs"), Text::new(IMPL_GENERIC))],
        Flagged::new(&["a.rs:11"]),
    ),
    Case::new(
        Title::new("boxed port"),
        &[Source::new(Located::new("a.rs"), Text::new(BOXED))],
        Flagged::new(&["a.rs:9"]),
    ),
];

#[test]
fn ports_called_per_element_are_flagged() {
    assert_eq!(Sources::mismatches::<PortBatching>(&CASES), Lines::default());
}
