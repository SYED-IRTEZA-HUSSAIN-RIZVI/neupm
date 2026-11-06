# NEUPM Lockfile System Design

## 1. Overview

NEUPM uses `neupm-lock.json` to guarantee reproducible installations across environments and CI/CD pipelines.

The lockfile records the exact resolved version, the download URL, the SHA-512 cryptographic integrity hash, whether the dependency is direct or transitive, and its declared sub-dependencies.

---

## 2. Lockfile Format (Version 1)

Example `neupm-lock.json`:

```json
{
  "lockfile_version": 1,
  "packages": {
    "react": {
      "name": "react",
      "version": "18.3.1",
      "resolved": "https://registry.npmjs.org/react/-/react-18.3.1.tgz",
      "integrity": "sha512-wS+hAgJShR0KhEvPJArfuPVN1+Hz1t0Y6n5jLrGQbkb4urgPE/0Rve+1kMB1v/oWgHgm4WIcV+i7F2pTVj+2iQ==",
      "is_direct": true,
      "dependencies": {
        "loose-envify": "^1.1.0"
      }
    },
    "loose-envify": {
      "name": "loose-envify",
      "version": "1.4.0",
      "resolved": "https://registry.npmjs.org/loose-envify/-/loose-envify-1.4.0.tgz",
      "integrity": "sha512-lyuxPGr/Wfhrlem2CL/UcnUc1zcqKAImBDzukY7Y5F/yQiNdko6+fRLevlw1HgMySw7f611UIY408EtxRSoK3Q==",
      "is_direct": false,
      "dependencies": {
        "js-tokens": "^3.0.0 || ^4.0.0"
      }
    },
    "js-tokens": {
      "name": "js-tokens",
      "version": "4.0.0",
      "resolved": "https://registry.npmjs.org/js-tokens/-/js-tokens-4.0.0.tgz",
      "integrity": "sha512-RdJUflcE3cUzKiMqQgsCu06FPu9UdIJO0beYbPhHN4k6apgJtifcoCtT9bcxOpYBtpD2kCM6Sbzg4CausW/PKQ==",
      "is_direct": false
    }
  }
}
```

---

## 3. Atomic Serialization

Lockfiles are serialized with two-space indentation and a trailing newline, written through `utils::atomic_write::atomic_write`. This prevents corrupt, zero-byte lockfiles if a process is terminated mid-write.

---

## 4. Frozen Lockfile Mode

When invoked with `--frozen-lockfile`:
```bash
neupm install --frozen-lockfile
```

The system performs the following checks:
1. `neupm-lock.json` must exist on disk.
2. The lockfile format version must be supported.
3. Every direct dependency declared in `package.json` must have a corresponding entry in `neupm-lock.json`.
4. If any package is missing, the installation immediately aborts with an error detailing the missing packages.
5. Neither `package.json` nor `neupm-lock.json` are modified during frozen installation.
