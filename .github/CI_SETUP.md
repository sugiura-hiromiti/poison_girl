# CI Setup

CI uses Nix as the source of truth.

## Local Verification

Run the same command used by pull-request CI:

```bash
nix flake check -L
```

If the ambient shell cannot run a command, use the flake dev shell:

```bash
nix develop
```

or, with direnv:

```bash
direnv exec . <command>
```

## Adding Checks

Add new CI behavior to `flake.nix` under `checks`. Avoid adding direct `cargo`,
`rustup`, `apt`, or Homebrew setup to workflow YAML.

## Current Check Ownership

- Rust toolchain: `fenix` in `flake.nix`
- Cargo builds/tests/docs: `crane` in `flake.nix`
- Security audit tool: `cargo-audit` in `devShells.default`
- Developer tools: `devShells.default`
- CI cache: `DeterminateSystems/magic-nix-cache-action`

GitHub Actions is responsible only for repository checkout, Nix installation,
Nix store caching, artifact upload, Pages deployment, and release creation.
