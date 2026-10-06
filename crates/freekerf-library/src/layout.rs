//! Where things live inside a library checkout.

use std::path::{Path, PathBuf};

/// Directory layout of a library checkout (the repository root).
#[derive(Debug, Clone)]
pub struct Layout {
    root: PathBuf,
}

impl Layout {
    /// Layout rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Repository root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `data/materials`: one TOML file per material, any sub-directory depth.
    pub fn materials_dir(&self) -> PathBuf {
        self.root.join("data").join("materials")
    }

    /// `data/machines`: one TOML file per machine profile.
    pub fn machines_dir(&self) -> PathBuf {
        self.root.join("data").join("machines")
    }

    /// `data/safety/hazards.toml`.
    pub fn hazards_file(&self) -> PathBuf {
        self.root.join("data").join("safety").join("hazards.toml")
    }

    /// `schema/v1`.
    pub fn schema_dir(&self) -> PathBuf {
        self.root.join("schema").join("v1")
    }

    /// Path relative to the root (falls back to the input when outside it).
    pub fn relative<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        let l = Layout::new("/r");
        assert_eq!(l.root(), Path::new("/r"));
        assert_eq!(l.materials_dir(), Path::new("/r/data/materials"));
        assert_eq!(l.machines_dir(), Path::new("/r/data/machines"));
        assert_eq!(l.hazards_file(), Path::new("/r/data/safety/hazards.toml"));
        assert_eq!(l.schema_dir(), Path::new("/r/schema/v1"));
        assert_eq!(l.relative(Path::new("/r/data/x.toml")), Path::new("data/x.toml"));
        assert_eq!(l.relative(Path::new("/other")), Path::new("/other"));
    }
}
