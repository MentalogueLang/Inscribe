# Inscribe 0.2.2

## 0.2.2 - 2026-03-19

## Highlights

- Fixed incremental cache invalidation across compiler updates by salting cache fingerprints with compiler identity.
- Resolved stale-cache typechecking failures on imported `.mlib` structs (for example `ParseResult` field access after syncing a newer Inscribe build).
- Added a versioned `.suture/mlib/<package>/<version>/` regression test for nested imported struct fields.

## Artifacts

- `inscribe-0.2.2-windows-x64.zip`
- `inscribe-0.2.2-linux-x64.tar.gz`
