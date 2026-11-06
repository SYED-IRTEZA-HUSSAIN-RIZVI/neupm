# NEUPM Technical Debt & Architectural Roadmap

**Date:** 2026-11-07  
**Scope:** Architecture sustainability, future roadmap, known limitations, and optimization priorities.

---

## 1. Resolved Technical Debt

The recent refactoring and engineering milestones resolved several significant items of technical debt:

| Previous Debt Item | Resolution |
|---|---|
| Monolithic single-file logic | Split into modular crates: `cli`, `commands`, `core`, `registry`, `archive`, `installer`, `manifest`, `lockfile`, `resolver`, `security`, `utils`. |
| Hardcoded root directory assumption (`package/`) | Two-pass dynamic archive root detection via `find_archive_root()`, supporting `@types/node` (`node/`) and standard npm archives. |
| Single package installation only | Implemented recursive dependency resolution (`ResolutionEngine`) and DAG construction (`DependencyGraph`). |
| Missing lockfile | Implemented `neupm-lock.json` model, atomic serialization, and `--frozen-lockfile` validation. |
| Incomplete CLI commands | Completed `install`, `remove`, `update`, `init`, `doctor`, `audit`, and `list` commands. |

---

## 2. Identified Open Technical Debt & Future Milestones

### DEBT-01: npm `package-lock.json` v3 Interoperability
- **Status:** Documented Limitation
- **Description:** NEUPM currently writes `neupm-lock.json` (Format Version 1), which is optimized for deterministic Rust parsing. Full bi-directional interoperability with npm's complex `package-lock.json` v3 schema (including `packages["node_modules/..."]` flat keys) remains to be implemented.
- **Priority:** Phase 2 Milestone

### DEBT-02: Flat vs. Nested `node_modules` Placement
- **Status:** Documented Limitation
- **Description:** NEUPM currently places all resolved dependencies at the top level of `node_modules` (`node_modules/<pkg>` or `node_modules/@scope/<pkg>`). If two packages require conflicting major versions of the same dependency, npm nesting (`node_modules/A/node_modules/B`) or pnpm symlinked virtual stores will be required.
- **Priority:** High

### DEBT-03: Global Content-Addressable Storage (CAS)
- **Status:** Planned in `src/future/`
- **Description:** Packages are currently downloaded and extracted directly into the project's staging directory. A global store (like pnpm or cargo cache at `~/.neupm/store`) with hardlinking would eliminate redundant downloads and save disk space across multiple projects.
- **Priority:** Performance Milestone

### DEBT-04: Streaming Tarball Processing
- **Status:** Optimization
- **Description:** Currently, tarballs are downloaded into memory buffers (`Vec<u8>`) before hashing and extraction. For very large packages (>50MB), streaming through a hasher into the decompressor will reduce peak memory consumption.
- **Priority:** Medium

### DEBT-05: Concurrent Downloads with Bounded Worker Pool
- **Status:** Implemented in `src/future/parallel/` (experimental); needs promotion to core
- **Description:** Dependencies in `ResolutionResult` are downloaded sequentially. A bounded `tokio::sync::Semaphore` worker pool will enable safe, high-speed concurrent downloads without exceeding HTTP rate limits.
- **Priority:** High
