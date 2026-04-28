# spec-forge-cli

npm wrapper for the `spec-forge-cli` CLI.

## Install

```bash
npm install -g @cli-forge-bin/spec-forge-cli
```

The matching native binary ships in a per-platform npm package that is selected
automatically via `optionalDependencies`. No postinstall download required.

## Supported platforms

- darwin-arm64, darwin-x64
- linux-arm64, linux-x64
- win32-arm64, win32-x64

## Notes

- This package is a thin wrapper around the published native binaries.
- For command help, run `spec-forge-cli --help` or `spec-forge-cli help --format json`.
- In a source checkout, the full operator guide lives at [`../../README.md`](../../README.md), which resolves to `spec-forge-cli/README.md`.
