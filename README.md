# NEUPM — Next-Generation Native Package Manager

NEUPM is a high-performance native package manager written in Rust, designed from scratch for maximum correctness, security, speed, and developer ergonomic superiority.

## Key Capabilities

- **Strict Archive Security**: Immune to path traversal (`../`), symlink/hardlink escapes, absolute paths, UNC paths, and Windows drive-letter injections.
- **Dynamic Root Detection**: Correctly extracts both standard npm packages (`react`) and non-standard root layouts (`@types/node`, DefinitelyTyped).
- **Staged Installation with Atomic Rollback**: Unsuccessful installations never corrupt or delete existing packages.
- **Scoped Package Support**: Native handling for `@scope/package-name` with correct directory hierarchies.
- **SHA-512 Verification**: Cryptographic package verification prior to extraction.
- **Atomic Manifest Updates**: Safe persistence of `package.json` dependencies.
- **Modular Layered Architecture**: Clean separation between CLI, core, registry, resolver, archive, installer, security, manifest, and future experimental layers.

## Architecture Overview

```
src/
├── main.rs            # Minimal CLI entry point
├── lib.rs             # Public library export
├── app.rs             # Application runner & command dispatch
├── cli/               # Command-line interface definitions & formatting
├── commands/          # Subcommand implementations (add, install, etc.)
├── core/              # Core domain models (package, context, error)
├── registry/          # Registry HTTP client & npm protocol adapter
├── archive/           # Safe archive inspection & extraction engine
├── installer/         # Staged installation, replacement & rollback
├── manifest/          # package.json management & atomic writer
├── security/          # Integrity checking & archive security rules
├── utils/             # Atomic write, filesystem & validation utilities
└── future/            # Experimental area (AI, parallel, distributed, caching)
```

## Quick Start

### Building from Source

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo build --release
```

### Adding a Package

```powershell
neupm add react
neupm add @types/node
neupm add @types/react
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
