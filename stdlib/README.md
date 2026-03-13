# Mantelogue Standard Library

This tree is intentionally split in two:

- `stdlib/core/`
  Pure Mantelogue source.
  If something can be expressed without direct operating-system access, it should live here.

- `stdlib/runtime/`
  Boundary declarations for services the compiler/runtime must provide.
  File I/O, process control, clocks, randomness, environment access, networking, and platform ABI hooks belong on this side.

Current rule of thumb:

- Needs the host system or target ABI directly: embed it in the compiler/runtime.
- Pure data transforms and control-flow helpers: write it in Mantelogue.

The files here are kept intentionally small and conservative so they match the subset of Mantelogue that `inscribe` already parses and lowers today.
