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
digest issue. Failure comments focus on diagnosis: likely cause, evidence,
impact, next actions, and experiment ideas. The daily digest asks the model for
structured JSON and renders it through a fixed Markdown layout with a metric
table and collapsible evidence sections. This keeps the digest readable while
still surfacing patterns and ideas instead of only listing activity counts. The
model is selected by `AI_DOCTOR_MODEL` in the workflow. It does not define build
behavior.

Most prompt construction and markdown rendering lives in
`.github/scripts/ai-doctor.js`; workflow YAML should only connect GitHub
Actions steps and permissions.

### `ai-second-review.yml`

Pull request second-opinion workflow using GitHub Models through
`actions/ai-inference`. It runs from `pull_request_target`, checks out only the
base repository workflow code, reads the PR diff through the GitHub API, and
creates or updates a single PR comment. The default second-review model is
`AI_SECOND_REVIEW_MODEL=azureml-deepseek/DeepSeek-V3-0324`.

The review is intentionally a diff-only design and implementation review. It is
not the final authority for `unsafe` safety. Prompt construction, response
parsing, and comment rendering live in `.github/scripts/ai-second-review.js`.
Docs-only changes are gated without calling the model.

### `weekly-ai-doctor.yml`

Weekly architecture and CI trend review. It runs Monday 00:30 JST, collects the
previous seven-day JST window, asks GitHub Models for a structured architecture
review, and opens or updates a weekly issue. The default model is
`AI_WEEKLY_ARCH_MODEL=openai/gpt-5`.

### AI model roles

- `AI_SECOND_REVIEW_MODEL`: DeepSeek second opinion for PR diffs.
- `AI_DOCTOR_MODEL`: CI failure diagnosis and daily digest.
- `AI_LIGHT_MODEL`: reserved for lightweight classification or free-limit
  spreading.
- `AI_WEEKLY_ARCH_MODEL`: weekly architecture and technical-debt trend review.

## Maintenance Rule

When adding or changing build, lint, test, audit, or documentation behavior,
update `flake.nix` first. Workflow YAML should stay a thin invocation layer
around Nix.
