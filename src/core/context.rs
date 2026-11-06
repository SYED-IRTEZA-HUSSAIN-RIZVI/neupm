use std::path::{Path, PathBuf};

/// Global execution context for NEUPM operations.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub working_dir: PathBuf,
    pub node_modules_dir: PathBuf,
    pub manifest_path: PathBuf,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl ExecutionContext {
    pub fn new(working_dir: PathBuf) -> Self {
        let node_modules_dir = working_dir.join("node_modules");
        let manifest_path = working_dir.join("package.json");
        Self {
            working_dir,
            node_modules_dir,
            manifest_path,
        }
    }

    pub fn node_modules(&self) -> &Path {
        &self.node_modules_dir
    }

    pub fn manifest(&self) -> &Path {
        &self.manifest_path
    }
}
