//! Lockfile system for reproducible NEUPM installations.
//!
//! NEUPM generates `neupm-lock.json` — a deterministic, human-readable
//! lockfile that records every installed package's resolved version,
//! tarball URL, integrity hash, and dependency relationships.
//!
//! Design notes:
//! - Format is NEUPM-native JSON (not npm `package-lock.json` v3).
//!   Full npm lockfile compatibility requires implementing the full v3
//!   schema and is tracked as a future milestone.
//! - The lockfile is written atomically via a temp-file rename.
//! - `neupm install` reads the lockfile when present and validates it
//!   against `package.json`. With `--frozen-lockfile`, any mismatch fails.

pub mod model;
pub mod reader;
pub mod validator;
pub mod writer;

pub use model::{LockEntry, Lockfile};
pub use reader::read_lockfile;
pub use validator::{LockfileValidationResult, validate_lockfile_against_manifest};
pub use writer::write_lockfile;
