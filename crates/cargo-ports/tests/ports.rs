use std::path::{Path, PathBuf};

use cargo_ports::{Cargo, CargoFailure, Recorded, Toolchain, cargo_ops};
use file_ports::Bytes;
use port_core::{Calls, Run};

#[test]
fn recorded_metadata_answers_for_its_manifests_only() {
    let recorded: Recorded = [(PathBuf::from("ws/Cargo.toml"), Bytes::from("{}"))].into_iter().collect();
    let run = Run::root();
    let answered = run.port(recorded).metadata(&[PathBuf::from("ws/Cargo.toml"), PathBuf::from("other/Cargo.toml")]);
    assert_eq!(answered.first().and_then(|answer| answer.as_ref().ok()), Some(&Bytes::from("{}")));
    assert!(matches!(
        answered.get(1),
        Some(Err(CargoFailure::Unrecorded { manifest })) if manifest == Path::new("other/Cargo.toml")
    ));
    assert_eq!(run.finish().snapshot().calls::<cargo_ops::Metadata>(), Calls::new(1));
}

#[test]
fn the_toolchain_describes_a_workspace_and_refuses_a_manifest_that_is_not_there() {
    let run = Run::root();
    let described = run.port(Toolchain).metadata(&[
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/.workspace/Cargo.toml"),
        Path::new("/no/such/workspace/Cargo.toml").to_path_buf(),
    ]);
    let lengths: Vec<Option<usize>> =
        described.iter().map(|read| read.as_ref().ok().map(|bytes| bytes.as_ref().len())).collect();
    assert_eq!(lengths.len(), 2);
    assert_ne!(lengths.first(), Some(&Some(0)));
    assert_ne!(lengths.first(), Some(&None));
    assert_eq!(lengths.get(1), Some(&None));
    assert_eq!(run.finish().snapshot().calls::<cargo_ops::Metadata>(), Calls::new(1));
}
