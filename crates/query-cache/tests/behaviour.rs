use std::io;
use std::path::Path;

use query_cache::{Entry, Origin, Table};
use test_failure::Failed;

#[test]
fn a_record_table_spells_its_statements_and_a_query_binds_only_what_it_takes() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let sections: Vec<Vec<Vec<String>>> = serde_json::from_slice(&std::fs::read(found)?)?;
    let [header, shapes @ ..] = sections.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let Some([statements, refusals, newline, entry, bound @ ..]) = header.first().map(Vec::as_slice) else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let spelled: Vec<String> = shapes
        .iter()
        .map(|shape| {
            let [named, columns @ ..] = shape.as_slice() else {
                return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
            };
            let [name, overwritten] = named.as_slice() else {
                return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
            };
            let built = columns.iter().try_fold(Table::named(name), |growing, row| match row.as_slice() {
                [column, origin] => Ok(growing.column(column, origin.parse::<Origin>()?)),
                _ => Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData))),
            })?;
            Ok(built
                .statements()
                .into_iter()
                .chain([Table::named(name).overwrite(overwritten)])
                .collect::<Vec<String>>())
        })
        .collect::<Result<Vec<Vec<String>>, Failed>>()?
        .into_iter()
        .flatten()
        .collect();
    let described = serde_json::from_str::<Entry<String>>(entry)?;
    let refused: Vec<String> = bound
        .iter()
        .map(|count| count.parse::<usize>())
        .collect::<Result<Vec<usize>, _>>()?
        .into_iter()
        .filter_map(|count| match described.bound(count) {
            Err(refusal) => Some(refusal.to_string()),
            Ok(()) => None,
        })
        .collect();
    assert_eq!(
        [[spelled.join(newline), newline.clone()].concat(), [refused.join(newline), newline.clone()].concat()],
        [std::fs::read_to_string(statements)?, std::fs::read_to_string(refusals)?]
    );
    Ok(())
}
