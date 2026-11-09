use std::path::{Path, PathBuf};

use file_ports::Absent;
use lint_placement::Placement;

struct Placed {
    placement: Placement,
    file: PathBuf,
}

impl Placed {
    fn of(pair: &[&str]) -> Option<Placed> {
        Some(pair).filter(|candidate| candidate.len() == 2).and_then(|two| two.first().zip(two.last())).map(
            |(package, file)| Placed {
                placement: Placement::within(&Absent, Path::new(package), Path::new(file)),
                file: PathBuf::from(file),
            },
        )
    }

    fn module(pair: &[&str]) -> Option<String> {
        Self::of(pair).map(|placed| {
            placed.placement.module().segments().iter().map(ToString::to_string).collect::<Vec<String>>().join("::")
        })
    }

    fn crate_root(pair: &[&str]) -> Option<PathBuf> {
        Self::of(pair).map(|placed| placed.placement.crate_root().directory().to_path_buf())
    }

    fn crate_file(pair: &[&str]) -> Option<PathBuf> {
        Self::of(pair).map(|placed| placed.placement.crate_file(&placed.file).as_ref().to_path_buf())
    }
}

laws::table!(a_source_file_is_placed_in_the_module_its_path_names, Placed::module, [
    ["/p", "/p/src/main.rs"] => Some(String::new()),
    ["/p", "/p/src/lib.rs"] => Some(String::new()),
    ["/p", "/p/src/bin/tool.rs"] => Some(String::from("bin::tool")),
    ["/p", "/p/src/outer/inner.rs"] => Some(String::from("outer::inner")),
]);

laws::table!(a_file_outside_a_crate_directory_is_its_own_crate_root, Placed::crate_root, [
    ["/p", "/p/build.rs"] => Some(PathBuf::from("/p")),
    ["/p", "/p/loose.rs"] => Some(PathBuf::from("/p/loose.rs")),
    ["/p", "/p/build.rs/inner.rs"] => Some(PathBuf::from("/p/build.rs/inner.rs")),
]);

laws::table!(a_module_file_belongs_to_its_library_and_a_build_script_to_itself, Placed::crate_file, [
    ["/p", "/p/src/outer/inner.rs"] => Some(PathBuf::from("/p/src/lib.rs")),
    ["/p", "/p/build.rs"] => Some(PathBuf::from("/p/build.rs")),
    ["/p", "/p/src/main.rs"] => Some(PathBuf::from("/p/src/main.rs")),
]);
