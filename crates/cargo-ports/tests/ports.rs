use std::path::{Path, PathBuf};

use cargo_ports::{Cargo, CargoFailure, Recorded, Toolchain, cargo_ops};
use file_ports::Bytes;
use port_core::{Calls, Run};

#[test]
fn the_toolchain_describes_a_workspace_and_refuses_a_manifest_without_one() {
    let run = Run::root();
    let described = run
        .port(Toolchain)
        .metadata_many(&[Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"), PathBuf::from("/Cargo.toml")]);
    let lengths: Vec<Option<usize>> =
        described.iter().map(|read| read.as_ref().ok().map(|bytes| bytes.as_ref().len())).collect();
    assert_eq!(lengths.len(), 2);
    assert_ne!(lengths.first(), Some(&Some(0)));
    assert_ne!(lengths.first(), Some(&None));
    assert_eq!(lengths.get(1), Some(&None));
    assert_eq!(run.finish().snapshot().calls::<cargo_ops::Metadata>(), Calls::new(2));
}

#[test]
fn recorded_metadata_answers_for_its_manifests_only() {
    let run = Run::root();
    let recorded: Recorded = [(PathBuf::from("ws/Cargo.toml"), Bytes::from("{}"))].into_iter().collect();
    let cargo = run.port(recorded);
    assert_eq!(cargo.metadata(Path::new("ws/Cargo.toml")).ok(), Some(Bytes::from("{}")));
    assert!(matches!(
        cargo.metadata(Path::new("other/Cargo.toml")),
        Err(CargoFailure::Unrecorded { manifest }) if manifest == Path::new("other/Cargo.toml")
    ));
}
