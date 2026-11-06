# NEUPM Architecture Overview

## Design Goals
1. **Security & Sandboxing**: Untrusted archives cannot escape staging directories or tamper with system files.
2. **Deterministic & Resilient**: Installations are staged, verified, and atomically moved into place.
3. **Modularity**: Small, reusable modules with minimal inter-dependencies.
4. **Future-Proof**: Experimental capabilities (AI, parallel graphs, distributed storage) are cleanly decoupled in `src/future/`.

## Layer Hierarchy

```
┌──────────────────────────────────────────────┐
│                  CLI Layer                   │ (arguments, output, progress)
└──────────────────────┬───────────────────────┘
                       │
┌──────────────────────▼───────────────────────┐
│               Commands Layer                 │ (add, install, remove, etc.)
└──────────────────────┬───────────────────────┘
                       │
┌──────────────────────▼───────────────────────┐
│              Core & Context                  │ (package spec, context, errors)
└───────┬──────────────┬───────────────┬───────┘
        │              │               │
┌───────▼──────┐┌──────▼──────┐┌───────▼───────┐
│   Registry   ││   Archive   ││   Installer   │
│   (client,   ││ (extractor, ││ (staged,      │
│  npm adapter)││  security)  ││  rollback)    │
└───────┬──────┘└──────┬──────┘└───────┬───────┘
        │              │               │
┌───────▼──────────────▼───────────────▼───────┐
│            Manifest & Utilities              │ (package.json, atomic write, fs)
└──────────────────────────────────────────────┘
```
