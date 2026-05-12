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
impact, next actions, and experiment ideas. The daily digest covers previous-day
repository statistics, allowlisted general news that may affect the repository,
and one weekday lens artifact for weekly synthesis. It calls the news model only
when news candidates exist, the lens model only when that weekday lens has
deterministic signal cards, and the digest model only when CI failures, relevant
news, medium/high lens findings, CI/Nix changes, safety-sensitive Rust deltas,
source-without-test changes, or unusually broad activity justify synthesis.
Otherwise it renders deterministic Markdown without calling a model. The
model-backed and deterministic paths both use a fixed Markdown layout with a
metric table and collapsible evidence sections. It does not define build
behavior.

Most prompt construction and markdown rendering lives in
`.github/scripts/ai-doctor.js`; allowlisted news collection lives in
`.github/scripts/ai-news.js`. Workflow YAML should only connect GitHub Actions
steps and permissions.

### `ai-second-review.yml`

Pull request second-opinion workflow using GitHub Models through
`actions/ai-inference`. It runs from `pull_request_target`, checks out only the
base repository workflow code, reads the PR diff through the GitHub API, and
creates or updates a single PR comment. The default second-review model is
the repository variable `AI_SECOND_REVIEW_MODEL`, falling back to
`mistral-ai/codestral-2501`.

The review is intentionally a diff-only design and implementation review. It is
not the final authority for `unsafe` safety. Prompt construction, response
parsing, and comment rendering live in `.github/scripts/ai-second-review.js`.
Docs-only changes are gated without calling the model.

### `weekly-ai-doctor.yml`

Weekly architecture and CI trend review. It runs Monday 00:30 JST, collects the
previous seven-day JST window, reads the daily weekday lens artifacts embedded
in daily digest issues, cross-checks them against deterministic weekly signals,
and opens or updates a weekly issue. The model is the repository variable
`AI_WEEKLY_ARCH_MODEL`, falling back to `openai/gpt-4.1`. The weekly model is
called only when lens artifacts, CI failures, workflow/Nix/Cargo changes,
Rust safety/debt token deltas, source-without-test changes, or unusually broad
weekly activity create a real synthesis trigger.

### AI model roles

- `AI_SECOND_REVIEW_MODEL`: provider-neutral second opinion for PR diffs;
  default `mistral-ai/codestral-2501`.
- `AI_CI_FAILURE_MODEL`: CI failure diagnosis; default
  `openai/gpt-4.1-mini`.
- `AI_DAILY_NEWS_MODEL`: ranks allowlisted general news candidates for
  repository relevance; default `xai/grok-3`.
- `AI_DAILY_DIGEST_MODEL`: daily previous-day statistics, CI, news, and lens
  digest; default `openai/gpt-4.1-nano`.
- `AI_WEEKLY_LENS_MODEL`: first-layer weekday lens artifact generator for
  Monday, Tuesday, Wednesday, and Friday lenses; default
  `deepseek/deepseek-r1-0528`.
- `AI_WEEKLY_BOTTLENECK_MODEL`: Thursday bottleneck lens model; default
  `xai/grok-3`.
- `AI_WEEKLY_ARCH_MODEL`: second-layer weekly synthesis over daily lens
  artifacts and deterministic weekly signals; default `openai/gpt-4.1`.

Daily digest and weekly review use an enforced two-layer policy. The first
layer runs at most one weekday lens per daily digest target date:

- Monday: layer boundary health.
- Tuesday: CI signal quality.
- Wednesday: low-layer safety and policy.
- Thursday: project bottleneck.
- Friday: test gap and technical debt.
- Saturday and Sunday: no weekly lens.

The first layer receives deterministic signal cards and writes a compact JSON
artifact into the daily digest issue as a hidden marker. The second layer weekly
review reads those artifacts and does not need raw daily diffs or logs. General
news is collected from allowlisted feeds and ranked separately; the news model
is called only when candidates exist.

Before any `actions/ai-inference` step runs, the selected model ID is checked
against the GitHub Models catalog. Missing or unavailable models skip inference
and render the deterministic fallback path with the catalog failure reason.

## Maintenance Rule

When adding or changing build, lint, test, audit, or documentation behavior,
update `flake.nix` first. Workflow YAML should stay a thin invocation layer
around Nix.
