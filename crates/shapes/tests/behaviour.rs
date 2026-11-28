use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::{ascii, iter, mem};

use file_ports::{Files, Kind};
use fixture::{Cases, Debugged, Failed, Fixture, Properties};
use port_core::Run;
use proptest::prelude::any;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use serde::Deserialize;
use shapes::{Campaign, Input};

#[derive(Fixture, Deserialize, Debug)]
struct Sampled {
    drawn: NonZeroUsize,
    kept: NonZeroUsize,
}

#[test]
fn the_fuzz_generator_draws_the_workspaces_its_golden_lists() -> Result<(), Failed> {
    let drifted = Cases::<Sampled>::drifted(|case| {
        let mut runner = TestRunner::new(Properties::settled());
        let pool = iter::repeat_with(|| any::<Input>().new_tree(&mut runner).map(|tree| tree.current()))
            .take(case.row().drawn.get())
            .collect::<Result<Vec<Input>, _>>()?;
        let shapes = pool
            .iter()
            .map(mem::discriminant)
            .fold(Vec::new(), |seen, shape| if seen.contains(&shape) { seen } else { [seen, vec![shape]].concat() });
        let drawn = shapes.iter().flat_map(|shape| {
            pool.iter().filter(move |input| mem::discriminant(*input) == *shape).take(case.row().kept.get())
        });
        let rendered = drawn
            .map(|input| {
                let run = Run::root();
                let files = run.port(input.files());
                let placed = files
                    .entries(&Campaign::root())?
                    .into_iter()
                    .map(|path| match files.kind(&path) {
                        Kind::Directory => files.entries(&path),
                        Kind::File | Kind::Link | Kind::Missing => Ok(vec![path]),
                    })
                    .collect::<Result<Vec<Vec<PathBuf>>, _>>()?
                    .concat();
                placed
                    .into_iter()
                    .map(|path| {
                        let bytes = files.read(&path)?;
                        let text: String =
                            bytes.as_ref().iter().copied().flat_map(ascii::escape_default).map(char::from).collect();
                        Ok((path, text))
                    })
                    .collect::<Result<Vec<(PathBuf, String)>, Failed>>()
            })
            .collect::<Result<Vec<_>, Failed>>()?;
        Ok(Debugged(rendered).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
