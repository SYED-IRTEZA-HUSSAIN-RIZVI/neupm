# Changelog

All notable changes to NEUPM will be documented in this file.

## [0.2.0] - 2026-11-07

### Added
- **Semantic Version Resolution Engine:** Full support for semver syntax (`react@18.3.1`, `react@^18.0.0`, `react@~18.2.0`, `react@22`, wildcards `*`/`1.x`, space-separated comparators, `||` disjunctions, and dist-tags).
- **Recursive Transitive Dependency Resolution:** Automated breadth-first traversal of runtime dependencies with deduplication and cycle detection.
- **Dependency Graph & Topological Sort:** Kahn's topological sort ensuring dependencies are installed strictly prior to parent packages.
- **Lockfile Engine (`neupm-lock.json`):** Version 1 deterministic lockfile generation, parsing, and atomic updates.
- **Reproducible Installs & Frozen Mode:** `neupm install --frozen-lockfile` enforcing consistency between `package.json` and `neupm-lock.json`.
- **Complete CLI Command Suite:**
  - `neupm add`: installs package + transitive dependencies, updates manifest and lockfile.
  - `neupm install`: installs from lockfile or synchronizes missing dependencies.
  - `neupm remove`: cleanly removes dependency from `package.json`, `node_modules`, and lockfile.
  - `neupm update`: upgrades direct dependencies within semver constraints and updates lockfile.
  - `neupm init`: initializes standard `package.json` with project defaults.
  - `neupm doctor`: comprehensive health check of environment, manifest, lockfile, `node_modules`, and registry.
  - `neupm audit`: audits integrity hashes, transport security (HTTPS), and version constraints.
  - `neupm list`: displays tree of direct and transitive dependencies with declared vs installed versions.
- **Expanded Test Suite:** 77 passing automated unit and integration tests (68 unit + 9 integration).
- **Comprehensive Architecture Documentation:** Added audits, threat models, transaction designs, lockfile design, and benchmarks in `docs/architecture/`.

## [0.1.0] - 2026-11-06

### Added
- Modular layered Rust architecture (`cli`, `core`, `registry`, `archive`, `installer`, `manifest`, `security`, `utils`, `future`).
- Dynamic root detection for non-standard npm tarball layouts (`@types/node`).
- Strict archive security against path traversal, absolute paths, UNC paths, and symlink/hardlink attacks.
- Staged installation engine with automated rollback on failure.
- Scoped package support (`@scope/package-name`).
- Atomic manifest writer for `package.json`.
- Comprehensive unit and integration test suite.
