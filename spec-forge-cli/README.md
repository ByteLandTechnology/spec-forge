# spec-forge-cli

Native Rust CLI for operating the spec-forge YAML workflow.

## Common Commands

```bash
spec-forge-cli init --target . --request-title "My Spec"
spec-forge-cli resolve --target . --write --format json
spec-forge-cli apply --target . --stage intake --parameter problem_goal --value '"Ship the workflow"'
spec-forge-cli gate check --target . --stage intake --write
spec-forge-cli stage advance --target . --stage intake
spec-forge-cli ux validate --target .
```

## Output Formats

All structured commands support `--format yaml`, `--format json`, and
`--format toml`. Human help is available through `--help`; structured help is
available through `spec-forge-cli help`.

## Packaged Assets

The CLI embeds the workflow templates and UX contract from `assets/` at compile
time so packaged builds do not depend on files outside the crate.

## Release Automation

The npm release harness lives in this directory, while the GitHub Actions
workflow entrypoint lives at the repository root because this project is stored
inside a multi-skill repository.

```bash
npm ci
npm run release:rehearse
```

The rehearsal command builds all configured targets, generates the platform npm
packages, and runs `npm publish --dry-run` without creating tags, GitHub
Releases, or real npm publications.

Local rehearsal needs the same cross-build tools as CI: `cargo-zigbuild`,
`zig`, installed Rust targets, and `llvm-mingw` linkers for Windows targets.
The GitHub workflow installs the Windows linkers automatically through
`spec-forge-cli/.github/actions/setup-build-env`.

Before the first production CI release, run the one-time bootstrap prepublish:

```bash
npm run release:prepublish
```

If local npm authentication is missing, the helper runs `npm login`; open the
verification URL that npm prints and complete the browser verification before
continuing. Production releases are driven from `.github/workflows/release.yml`
on `main` and require npm trusted publishing entries for
`@cli-forge-bin/spec-forge-cli` plus each
`@cli-forge-bin/spec-forge-cli-<platform>` platform package.

Users should normally install with:

```bash
npm install -g @cli-forge-bin/spec-forge-cli
```

For clone-first installs from a released GitHub tag, run:

```bash
./scripts/install-current-release.sh
```

## Validation

```bash
cargo test --manifest-path spec-forge-cli/Cargo.toml
cargo fmt --manifest-path spec-forge-cli/Cargo.toml --check
cargo clippy --manifest-path spec-forge-cli/Cargo.toml -- -D warnings
cargo package --manifest-path spec-forge-cli/Cargo.toml --offline
```

## License

MIT
