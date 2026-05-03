# GitHub Actions Workflows

This repository keeps CI logic in `flake.nix`. GitHub Actions should install Nix
and call flake outputs instead of installing Rust tools or duplicating Cargo
commands in YAML.

## Active Workflows

### Shared Nix Setup

Nix installation and Magic Nix Cache setup are centralized in
`.github/actions/setup-nix/action.yml`.

`ci.yml` and `multi-platform-ci.yml` call the reusable
`.github/workflows/nix-flake-check.yml` workflow so the flake check command is
owned in one place.

### `ci.yml`

Runs on pushes and pull requests to `main` and `develop`.

The workflow executes:

```bash
nix flake check -L --show-trace
```

The Nix store is cached with `DeterminateSystems/magic-nix-cache-action`.
Cargo registry and `target` directories are not cached separately.

The checks are defined in `flake.nix` under `checks`:

- `workspace_metadata`
- `workspace_build`
- `workspace_clippy`
- `workspace_test`
- `workspace_doc`
- `workspace_fmt`
- `kernel_aarch64_check`
- `loader_aarch64_uefi_check`

### `docs.yml`

Runs on pushes to `main` and manual dispatch. It builds the `workspace_doc`
flake check and publishes the generated rustdoc to GitHub Pages.

### `release.yml`

Runs for `v*` tags and manual dispatch. It builds `packages.${system}.default`
from the flake and uploads the Nix output as the release artifact.

### `multi-platform-ci.yml`

Manual-only smoke workflow for checking the same flake on Linux and macOS
runners. It calls the shared flake-check workflow once for each runner. It is
intentionally not run for every pull request.

### `build-and-run.yml`

Manual-only dev shell smoke workflow. It verifies that Nix provides the expected
tools such as Cargo, Rust, QEMU, and binutils.

### `batch-ai-doctor.yml`

Independent scheduled PR health workflow. It checks `ci.yml` runs by PR head
SHA, opens or updates CI failure issues with GitHub Models, and creates a daily
digest issue. The model is selected by `AI_DOCTOR_MODEL` in the workflow. It
does not define build behavior.

## Maintenance Rule

When adding or changing build, lint, test, audit, or documentation behavior,
update `flake.nix` first. Workflow YAML should stay a thin invocation layer
around Nix.
