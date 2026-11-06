pub mod dependency_manager;
pub mod package_json;
pub mod writer;

pub use dependency_manager::update_dependency_in_manifest;
pub use package_json::PackageJson;
pub use writer::save_manifest;
