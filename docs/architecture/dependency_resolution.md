# NEUPM Dependency Resolution Engine

## 1. Overview

NEUPM implements a robust, deterministic, recursive dependency resolution engine in `src/resolver/`.

When a package is added or installed:
```bash
neupm add react@18.3.1
```
The engine resolves the entire tree of runtime dependencies prior to initiating any file downloads or filesystem modifications.

---

## 2. Architecture & Pipeline

```text
User Input: "react@18.3.1"
      │
      ▼
[ParsedSpec::parse]  ───▶  name: "react", version_req: Some("18.3.1")
      │
      ▼
[ResolutionEngine::resolve]
      │
      ├─▶ 1. Fetch metadata from registry (`NpmRegistryAdapter`)
      ├─▶ 2. Select version (`version_selector::select_version`)
      │      - Dist-tag lookup ("latest", "next", "beta")
      │      - Semver candidate evaluation (`semver_util`)
      │      - Support for ^, ~, bare numbers (22), wildcards (*, 1.x), || disjunctions
      ├─▶ 3. Extract version metadata & dependencies
      ├─▶ 4. Queue runtime & optional dependencies (BFS queue)
      └─▶ 5. Prevent duplicate fetches with `enqueued` set
      │
      ▼
[DependencyGraph]
      │
      ├─▶ Build Directed Acyclic Graph (nodes = packages, edges = dependencies)
      ├─▶ Detect cycles
      └─▶ Kahn's Topological Sort:
          - Dependencies appear before the packages that depend on them
          - Alphabetical sorting for deterministic tie-breaking
      │
      ▼
[ResolutionResult]
      │
      └─▶ Ordered list of `ResolvedPackage` (metadata, is_direct flag)
```

---

## 3. Supported Version Range Syntax

NEUPM supports the standard npm semver selectors:

| Syntax | Example | Resolved Semantics |
|---|---|---|
| **Exact** | `18.3.1` | Matches only `18.3.1` |
| **Caret** | `^18.0.0` | `>=18.0.0, <19.0.0` |
| **Tilde** | `~18.2.0` | `>=18.2.0, <18.3.0` |
| **Bare Major** | `22` | Normalized to `^22` (`>=22.0.0, <23.0.0`) |
| **Wildcard** | `*`, `x`, `X` | Matches any version `>=0.0.0` |
| **Partial Wildcard** | `1.x` | Normalized to `^1` |
| **Minor Wildcard** | `1.2.x` | Normalized to `~1.2` |
| **Disjunction** | `^3.0.0 \|\| ^4.0.0` | Matches any version satisfying either range |
| **Compound Comparator** | `>=1.0.0 <2.0.0` | Matches intersection of both predicates |
| **Dist-Tag** | `latest`, `beta` | Resolves dist-tag directly from registry JSON |

---

## 4. Cycle Detection & Determinism

Cycle detection is built into `DependencyGraph::installation_order()`. If a circular dependency exists between packages, the algorithm detects that the resolved list length is smaller than the total graph size and bails with an error identifying the packages involved.

Independent nodes with in-degree 0 are sorted alphabetically to ensure reproducible results regardless of network delivery order.
