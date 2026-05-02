# Nix-Centered CI

The repository now treats `flake.nix` as the source of truth for CI.

## What Moved Into Nix

- Rust nightly toolchain selection
- Package build arguments
- Format, clippy, test, and doc checks
- QEMU/binutils/dev tools in the development shell

## What Remains In GitHub Actions

- Installing Nix
- Restoring and saving the Nix store cache
- Running `nix flake check -L`
- Publishing docs and release artifacts
- Manual platform/dev-shell smoke workflows

## Why

Keeping check definitions in one place reduces drift between local development
and CI. It also removes duplicated Rust setup, Cargo cache configuration,
platform-specific package installation, and stale crate path references from
workflow YAML.

## Updating CI

Change `flake.nix` first. A pull request should not need to edit workflow YAML
unless it changes GitHub-specific behavior such as triggers, permissions,
artifacts, Pages, or releases.
