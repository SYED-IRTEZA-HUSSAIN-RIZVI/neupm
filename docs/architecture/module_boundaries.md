# Module Boundaries & Responsibilities

| Module | Single Responsibility | Prohibited Activities |
|---|---|---|
| `cli` | Parsing user inputs and formatting console output | Direct network requests or filesystem mutations |
| `commands` | Dispatching and orchestrating user intents | Low-level archive parsing or direct registry HTTP logic |
| `core` | Domain types (`PackageMetadata`, `PackageSpec`) | Terminal output or platform-specific syscalls |
| `registry` | Querying registry endpoints and downloading tarballs | Unpacking archives or modifying `node_modules` |
| `archive` | Safe tarball inspection, validation, and extraction | Updating `package.json` or contacting registries |
| `installer` | Coordinating staging, replacement, and rollback | Rendering CLI output directly |
| `manifest` | Parsing and serializing project manifests (`package.json`) | Downloading or extracting archives |
| `security` | Validating hashes, permissions, and archive paths | Managing UI state |
| `utils` | Reusable utilities (atomic writing, validation, filesystem) | Business domain decisions |
| `future` | Experimental features (AI, parallel, distributed, caching) | Modifying stable package installation behaviour |
