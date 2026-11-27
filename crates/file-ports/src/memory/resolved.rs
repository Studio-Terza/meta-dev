use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct Resolved(PathBuf);

impl Resolved {
    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    fn parent_of(resolved: PathBuf) -> PathBuf {
        match resolved.parent().map(Path::to_path_buf) {
            Some(parent) => parent,
            None => resolved,
        }
    }
}

impl Resolved {
    fn stepped(resolved: PathBuf, component: Component) -> PathBuf {
        match component {
            Component::CurDir => resolved,
            Component::ParentDir => Self::parent_of(resolved),
            kept => resolved.join(kept),
        }
    }
}

impl From<&Path> for Resolved {
    fn from(path: &Path) -> Resolved {
        Resolved(path.components().fold(PathBuf::new(), Self::stepped))
    }
}
