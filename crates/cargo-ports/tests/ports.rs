use std::path::{Path, PathBuf};

use cargo_ports::{Cargo, CargoFailure, Recorded};
use file_ports::Bytes;

#[test]
fn recorded_metadata_answers_for_its_manifests_only() {
    let recorded: Recorded = [(PathBuf::from("ws/Cargo.toml"), Bytes::from("{}"))].into_iter().collect();
    assert_eq!(recorded.metadata(Path::new("ws/Cargo.toml")).ok(), Some(Bytes::from("{}")));
    assert!(matches!(
        recorded.metadata(Path::new("other/Cargo.toml")),
        Err(CargoFailure::Unrecorded { manifest }) if manifest == Path::new("other/Cargo.toml")
    ));
}
