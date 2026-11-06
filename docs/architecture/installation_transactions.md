# NEUPM Installation Transactions & Rollback Model

## 1. Overview

NEUPM protects the user's project from corrupted or partially-extracted packages using a multi-stage transactional pipeline with automatic rollback.

---

## 2. Transaction Workflow

```text
1. Planning & Resolution
   ├── Fully resolve dependency graph into a complete installation plan.
   └── Validate version constraints and detect dependency cycles.

2. Download & Cryptographic Verification
   ├── Download tarball into memory.
   └── Verify SHA-512 integrity before touching disk or decompressing.

3. Staging Area Extraction
   ├── Create temporary staging directory in target parent (same volume):
   │   `node_modules/.neupm-staging-<uuid>/`
   ├── Extract tarball while enforcing path traversal and symlink guards.
   └── Verify extracted package: ensure valid `package.json` is present.

4. Safe Replacement & Rollback
   ├── If target `node_modules/<pkg>` exists:
   │   ├── Rename existing package to `.neupm-backup-<pkg>`.
   │   └── Atomically rename staged directory to `node_modules/<pkg>`.
   │       ├── On Success: Remove `.neupm-backup-<pkg>`.
   │       └── On Error:   Automatically restore `.neupm-backup-<pkg>`
   │                       and clean staging area.
   └── If target does not exist:
       └── Rename staged directory directly to `node_modules/<pkg>`.

5. Manifest & Lockfile Commit
   ├── Atomically write `neupm-lock.json`.
   └── Atomically write `package.json` for direct dependencies.
```

---

## 3. Windows-Specific Rename Considerations

Windows NTFS enforces strict file locking semantics. To ensure atomic replacement:
1. The staging directory is created inside the package's parent directory (`parent_dir` in `node_modules`), ensuring that source and destination are on the same physical volume (preventing cross-volume move failures).
2. For scoped packages (e.g., `@types/node`), the scope directory (`node_modules/@types`) is created before the staging directory is initialized.
3. If a collision or lock prevents instant rename, the rollback handler restores the previous state from the backup directory.
