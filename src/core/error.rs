use std::fmt;

/// Domain error types for NEUPM operations.
#[derive(Debug)]
pub enum NeupmError {
    PackageNotFound(String),
    IntegrityMismatch { expected: String, actual: String },
    SecurityViolation(String),
    InvalidArchive(String),
    InstallationFailed(String),
}

impl fmt::Display for NeupmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageNotFound(pkg) => {
                write!(f, "Package '{pkg}' was not found in npm registry.")
            }
            Self::IntegrityMismatch { expected, actual } => {
                write!(f, "Integrity mismatch: expected {expected}, got {actual}")
            }
            Self::SecurityViolation(msg) => write!(f, "Security violation: {msg}"),
            Self::InvalidArchive(msg) => write!(f, "Invalid archive layout: {msg}"),
            Self::InstallationFailed(msg) => write!(f, "Installation error: {msg}"),
        }
    }
}

impl std::error::Error for NeupmError {}
