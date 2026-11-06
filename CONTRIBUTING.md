# Contributing to NEUPM

We welcome contributions to NEUPM! Please follow these guidelines:

## Architecture Principles
1. **Separation of Concerns**: Each module must have one clear, focused responsibility.
2. **Security First**: Never trust network or archive input. Validate all paths, checksums, and entry headers.
3. **Preserve Working Features**: Always run `cargo test` and verify that both standard packages and scoped packages install properly.
4. **Experimental Modules**: Keep all experimental code inside `src/future/` behind Cargo feature flags (`experimental`).

## Development Workflow
```powershell
# Format check
cargo fmt --check

# Compiler check
cargo check

# Run tests
cargo test

# Build release
cargo build --release
```
