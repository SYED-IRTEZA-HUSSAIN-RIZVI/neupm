# Changelog

All notable changes to NEUPM will be documented in this file.

## [0.1.0] - 2026-11-06

### Added
- Modular layered Rust architecture (`cli`, `core`, `registry`, `archive`, `installer`, `manifest`, `security`, `utils`, `future`).
- Dynamic root detection for non-standard npm tarball layouts (`@types/node`).
- Strict archive security against path traversal, absolute paths, UNC paths, and symlink/hardlink attacks.
- Staged installation engine with automated rollback on failure.
- Scoped package support (`@scope/package-name`).
- Atomic manifest writer for `package.json`.
- Comprehensive unit and integration test suite.
