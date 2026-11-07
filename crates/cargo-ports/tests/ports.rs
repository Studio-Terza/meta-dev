use std::path::{Path, PathBuf};

use cargo_ports::{Cargo, Toolchain, cargo_ops};
use port_core::{Calls, Run};

#[test]
fn the_toolchain_describes_a_workspace_and_refuses_a_directory_without_one() {
    let run = Run::root();
    let described =
        run.port(Toolchain).metadata(&[PathBuf::from(env!("CARGO_MANIFEST_DIR")), Path::new("/").to_path_buf()]);
    let lengths: Vec<Option<usize>> =
        described.iter().map(|read| read.as_ref().ok().map(|text| text.as_ref().len())).collect();
    assert_eq!(lengths.len(), 2);
    assert_ne!(lengths.first(), Some(&Some(0)));
    assert_ne!(lengths.first(), Some(&None));
    assert_eq!(lengths.get(1), Some(&None));
    assert_eq!(run.finish().snapshot().calls::<cargo_ops::Metadata>(), Calls::new(1));
}
