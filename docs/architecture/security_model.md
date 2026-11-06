# Security Model

NEUPM protects developers against malicious packages, registry tampering, and unsafe archive extraction:

## 1. Cryptographic Integrity
- Every package archive must include a valid SHA-512 integrity string published by the npm registry.
- Tarball bytes are decoded and verified using SHA-512 before any archive reader is invoked.

## 2. Archive Traversal & Injection Shield
Untrusted tar archives can contain malicious paths designed to overwrite arbitrary files outside `node_modules`. NEUPM implements multi-layered protections:
- **Path Component Sanitization**: Rejects any entry path containing `..` or null bytes.
- **Absolute & Drive Path Rejection**: Rejects root-relative (`/foo`) and Windows drive paths (`C:\foo`).
- **UNC Path Rejection**: Blocks network share references (`\\server\share`).
- **Symlink & Hardlink Blocking**: Prevents symbolic links and hard links from pointing outside the destination.
- **Canonical Boundary Check**: Every destination path is canonicalized to ensure containment within the temporary staging directory.

## 3. Atomic Replacement & Rollback
- Installations are extracted into an isolated staging directory on the same filesystem.
- If replacing an existing package, the old package is preserved as a backup until the new directory is verified in place.
- If moving fails or extraction encounters an error, the previous installation is safely rolled back.
