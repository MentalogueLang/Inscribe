# Release Notes

## 0.2.1 - 2026-03-18

### Highlights

- Added `inscribe emit mlib` and MLIB metadata/signature support for packaged libraries.
- Added resolver support for importing Suture-pulled `.mlib` libraries from `.suture/mlib`.
- Packaged `stdlib` into release artifacts and taught installed Inscribe builds to locate it on-device.

### Artifacts

- `inscribe-0.2.1-windows-x64.zip`
- `inscribe-0.2.1-linux-x64.tar.gz`

## 0.2.0 - 2026-03-15

### Highlights

- Added disk-backed artifact caching for graph/resolve/typeck/hir/mir stages.
- Added sandboxed execution (`inscribe run --sandbox`) backed by the MIR interpreter.
- Added `main -> string` output handling and richer debug line/source maps.
- Added HTTP runtime surface and determinism capability tracking updates.

### Artifacts

- `inscribe-0.2.0-windows-x64.zip`
- `inscribe-0.2.0-linux-x64.tar.gz`

## 0.1.1 - 2026-03-15

### Highlights

- Introduced stable HIR symbol ids and a canonical symbol table across lowering, pretty-printing, and MIR lowering.
- Ignored generated `.hir`/`.mir` artifacts and removed tracked copies from the repository.
- Updated the release workflow to build against the Inscribe workspace `Cargo.toml`.

### Artifacts

- `inscribe-0.1.1-windows-x64.zip`
- `inscribe-0.1.1-linux-x64.tar.gz`

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
