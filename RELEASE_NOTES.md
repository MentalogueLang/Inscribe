# Release Notes

## Unreleased

### Highlights

- Introduced stable HIR symbol ids and a canonical symbol table across lowering, pretty-printing, and MIR lowering.
- Ignored generated `.hir`/`.mir` artifacts and removed tracked copies from the repository.
- Updated the release workflow to build against the Inscribe workspace `Cargo.toml`.

## 0.1.0 - 2026-03-15

### Highlights

- Initial public Inscribe release.
- Added capability-aware MIR determinism tracking.
- Added stdin support and refreshed the calculator example.
- Added string helpers, enums, arrays, indexing, and `as` casts.
- Improved import handling, parse errors, and JSON parser example coverage.

### Artifacts

- `inscribe-0.1.0-windows-x64.zip`
- `inscribe-0.1.0-linux-x64.tar.gz`
