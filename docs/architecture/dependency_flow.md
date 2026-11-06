# Package Installation Flow

```
User: neupm add <package>
        │
        ▼
[cli::arguments] Parses CLI flags & package argument
        │
        ▼
[commands::add] Orchestrates addition workflow
        │
        ▼
[registry::npm] Queries metadata from npm registry
        │
        ▼
[registry::client] Downloads package archive tarball into memory
        │
        ▼
[security::integrity] Computes SHA-512 and matches against npm published digest
        │
        ▼
[installer::engine] Creates temporary staging directory (.neupm-staging-*)
        │
        ▼
[archive::extractor]
        ├── 1. Dynamic root detection (package/, node/, etc.)
        ├── 2. Path sanitization & link rejection
        └── 3. Unpack into staging directory
        │
        ▼
[archive::inspector] Verifies extracted package.json
        │
        ▼
[installer::replacement]
        ├── Backup existing package (if any) -> .neupm-backup-*
        ├── Move staging directory to final target
        └── On failure: Rollback from backup!
        │
        ▼
[manifest::package_json] Updates package.json atomically via temporary file
        │
        ▼
[cli::output] Emits clean, formatted installation confirmation
```
