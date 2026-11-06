# NEUPM Security Findings & Threat Model

**Audit Date:** 2026-11-07  
**Scope:** Tarball extraction, integrity verification, path validation, filesystem operations, and network security.

---

## 1. Threat Model & Security Posture

Package managers execute untrusted input downloaded over the Internet. An attacker controlling an npm package tarball can craft malicious archives designed to overwrite system files, escape working directories, or execute arbitrary code.

NEUPM applies defense-in-depth principles across four security layers:
1. **Transport Security**
2. **Cryptographic Integrity Verification**
3. **Archive Extraction Security**
4. **Filesystem Transaction Safety**

---

## 2. Security Findings & Mitigations

### SEC-01: Archive Path Traversal (Zip Slip / Tar Slip)
- **Severity:** Critical
- **Threat:** Tarball entries containing `../`, leading slashes (`/etc/passwd`), or nested escape sequences could write files outside the target directory.
- **NEUPM Mitigation:**
  - Multi-layer validation in `security::archive_security::validate_entry_path`.
  - Rejection of `Component::ParentDir`, `Component::RootDir`, and `Component::Prefix`.
  - Canonicalization check: every extracted destination directory and file path must have its canonical parent verified to start with the canonical staging directory.
  - Rejection of null bytes (`\0`).

### SEC-02: Windows Drive Letter & UNC Path Injection
- **Severity:** High
- **Threat:** On Windows, entries such as `C:\Windows\system32\malicious.dll` or `\\attacker\share\payload` bypass standard Unix path traversal checks.
- **NEUPM Mitigation:**
  - Explicit checks reject paths starting with ASCII alphabetic drive letters followed by `:` (e.g., `C:`).
  - Explicit checks reject UNC paths starting with `\\` or `//`.

### SEC-03: Symbolic Link and Hard Link Attacks
- **Severity:** High
- **Threat:** An archive entry could create a symlink to an arbitrary system file (e.g., `package/link -> /etc/passwd`), and a subsequent entry could write through the symlink to overwrite the target.
- **NEUPM Mitigation:**
  - All symlinks (`entry.header().entry_type().is_symlink()`) and hard links (`entry.header().entry_type().is_hard_link()`) are explicitly detected and rejected with an error prior to extraction.

### SEC-04: Integrity Verification Bypass & Tampering
- **Severity:** Critical
- **Threat:** In-flight network tampering or cache poisoning could serve modified tarball bytes not matching registry signatures.
- **NEUPM Mitigation:**
  - Mandatory SHA-512 verification (`verify_sha512_integrity`) executed on raw downloaded bytes before any decompression or extraction.
  - Verification parses the npm `sha512-` Base64 string, computes `sha2::Sha512`, and performs byte-for-byte comparison.

### SEC-05: Transport Security & Insecure Protocol Rejection
- **Severity:** Medium
- **Threat:** Packages fetched over unencrypted HTTP are subject to eavesdropping and man-in-the-middle manipulation.
- **NEUPM Mitigation:**
  - Registry adapter defaults to `https://registry.npmjs.org`.
  - `neupm audit` actively scans lockfile URLs and alerts on any non-HTTPS entries.

### SEC-06: Arbitrary Lifecycle Script Execution Policy
- **Severity:** High
- **Threat:** Malicious npm packages frequently execute arbitrary code during `preinstall` or `postinstall` lifecycle scripts.
- **NEUPM Mitigation:**
  - NEUPM does not automatically execute arbitrary lifecycle scripts during installation.
  - Scripts remain inactive by default, preventing drive-by remote code execution attacks from compromised dependencies.

---

## 3. Summary of Test Coverage

| Threat Vector | Test Function | Result |
|---|---|---|
| Path Traversal (`../evil.js`) | `security::archive_security::tests::test_rejects_path_traversal` | Passed |
| Absolute Paths (`/etc/passwd`) | `security::archive_security::tests::test_rejects_absolute_and_drive_paths` | Passed |
| Windows Drive Letters (`C:\...`) | `security::archive_security::tests::test_rejects_absolute_and_drive_paths` | Passed |
| UNC Network Paths (`\\server\...`) | `security::archive_security::tests::test_rejects_unc_paths` | Passed |
| Symlink Injection | `archive::tests::test_reject_symlinks` | Passed |
| Staging Escape | `archive::tests::test_no_files_escape_staging` | Passed |
| SHA-512 Tampering | `security::integrity::tests::test_tampered_integrity` | Passed |
| SHA-512 Valid Signature | `security::integrity::tests::test_valid_integrity` | Passed |
