# Inscribe

Inscribe is the compiler toolchain for the Mentalogue Language.

## Status

`0.1.0` is the first tracked Inscribe release. Release notes live in
[RELEASE_NOTE.md](RELEASE_NOTE.md) for the current release and
[RELEASE_NOTES.md](RELEASE_NOTES.md) for the full history.

## Build

Use Cargo to build the CLI locally:

```powershell
cargo build -p inscribe-cli --release
```

## Usage

```powershell
cargo run -p inscribe-cli -- check examples/json_parser/main.mtl
cargo run -p inscribe-cli -- build examples/json_parser/main.mtl
cargo run -p inscribe-cli -- run examples/json_parser/main.mtl
```

Tip: you can install via `mntpack sync MentalogueLang/Inscribe -g -r auto`, though it is preferred to install Inscribe through [Stratum](https://github.com/MentalogueLang/Stratum) when available.

## VS Code Extension

A Mentalogue VS Code extension is included at `tools/vscode-mentalogue`.

Features:

- `.mtl` and `.mlib` syntax highlighting
- IntelliSense from workspace `.mtl` and `.mlib` symbols (including `.suture` packages when present)
- Periodic `inscribe check` diagnostics on the active `.mtl` file

To test it locally:

1. Open `tools/vscode-mentalogue` in VS Code.
2. Press `F5` to launch an Extension Development Host.
3. Open a Mentalogue project and edit `.mtl` files.

## Check An Example

```powershell
cargo run -p inscribe-cli -- check examples/json_parser/main.mtl
```

## Releases

GitHub releases are created manually from Actions so they do not run on every
commit.

The `release.yml` workflow builds x64 binaries for:

- Windows (`x86_64-pc-windows-msvc`)
- Linux (`x86_64-unknown-linux-gnu`)

Before running the workflow, update:

- [RELEASE_NOTE.md](RELEASE_NOTE.md) with the notes for the release being cut
- [RELEASE_NOTES.md](RELEASE_NOTES.md) with the same notes appended to history

Then run the `Release` workflow from GitHub Actions and pass the version, such
as `0.1.0`.
