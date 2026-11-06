//! Semantic version resolution and dependency graph construction.
//!
//! This module resolves package version requirements against npm registry
//! metadata, selects compatible versions, and builds the dependency graph
//! needed for a complete installation plan.

pub mod dependency_graph;
pub mod package_spec;
pub mod resolution_engine;
pub mod semver_util;
pub mod version_selector;

pub use package_spec::ParsedSpec;
pub use resolution_engine::{ResolutionEngine, ResolutionResult, ResolvedPackage};
