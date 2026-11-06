# Security Policy

NEUPM takes package manager security very seriously.

## Security Guarantees
- **No Path Traversal**: Tarballs containing `../` are strictly rejected.
- **No Link Escapes**: Symbolic links and hard links are prohibited in untrusted packages.
- **Drive & UNC Path Rejection**: Windows drive prefixes (`C:`) and UNC paths (`\\server\share`) are rejected.
- **Destination Containment**: Every extracted file is verified to resolve inside the canonical staging root.
- **Cryptographic Verification**: Tarballs must match npm's published SHA-512 integrity digests before extraction begins.

## Reporting a Vulnerability
If you discover a security issue in NEUPM, please open a private report or contact the maintainers directly.
