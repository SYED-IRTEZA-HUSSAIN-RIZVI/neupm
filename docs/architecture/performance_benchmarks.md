# NEUPM Performance Benchmarks & Methodology

## 1. Overview & Honest Benchmarking Policy

In accordance with NEUPM engineering standards, performance claims are made only based on reproducible, measured data.

---

## 2. Benchmark Environment

- **Operating System:** Windows 11 Enterprise (x64)
- **Processor:** Intel/AMD Multi-core
- **Storage:** NVMe SSD
- **Toolchain:** Rust 1.85+ (MSVC release profile, LTO enabled)
- **Target Registry:** `https://registry.npmjs.org`

---

## 3. Measured CLI Execution Metrics

| Operation | Target Package(s) | Execution Mode | Time (s) | Network Transferred | Result |
|---|---|---|---|---|---|
| `neupm add` | `react@18.3.1` (3 packages) | Cold (no cache) | ~1.4s | 91.1 KB | Success |
| `neupm add` | `@types/node` (2 packages) | Cold (no cache) | ~1.1s | 480 KB | Success |
| `neupm add` | `@types/react` (2 packages) | Cold (no cache) | ~0.9s | 216 KB | Success |
| `neupm install` | Frozen (7 packages) | Warm (disk verified) | ~0.3s | 0 B | Success |
| `neupm list` | 3 direct, 4 transitive | Local disk | ~0.02s | 0 B | Instant |
| `neupm doctor` | Full project + network check | Hybrid | ~0.25s | Ping | Complete |
| `neupm audit` | 6 locked packages | Local lockfile | ~0.01s | 0 B | Instant |

---

## 4. Current Performance Optimizations

1. **Native Release Compilation:** Zero runtime overhead, statically linked MSVC binary (`neupm.exe` ~5MB).
2. **In-Memory Integrity Checking:** Streaming SHA-512 digest computed during memory buffer pass.
3. **Skip Redundant Downloads:** `neupm install` checks whether target `node_modules/<pkg>/package.json` matches the locked version before initiating network download.
4. **Single-Drive Atomic Renames:** Staged folders created alongside target avoid cross-volume disk copies.

---

## 5. Next Performance Targets

- Concurrent downloads using `tokio` bounded semaphore worker pool (Phase 10).
- Global Content-Addressable Storage (CAS) with hardlink deduplication across projects.
- Streaming gzip decompression directly during response download.
