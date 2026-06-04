# CI integration

Anchor Sentinel fits into CI as either a GitHub Action
(one-line) or a raw CLI step (more control). Both produce SARIF
output that GitHub Code Scanning understands.

## GitHub Action (recommended)

The action is vendored at
`.github/actions/scan` in this repo. Reference it with
`uses: Eniyanyosuva/anchor-sentinel/.github/actions/scan@main`.

### Minimal example

```yaml
name: Security Scan
on: [push, pull_request]
permissions:
  contents: read
  security-events: write
jobs:
  sentinel:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Anchor project
        run: anchor build
      - uses: Eniyanyosuva/anchor-sentinel/.github/actions/scan@main
        with:
          fail-on-severity: high
```

This will:
1. Run `cargo install anchor-sentinel --locked` on the runner.
2. Run `sentinel scan . --format sarif > sentinel.sarif`.
3. Upload `sentinel.sarif` to GitHub Code Scanning.
4. Exit 1 if any finding at or above `high` is present.

Findings appear as PR annotations (on the line in the source where
the bug lives) and in the **Security** tab.

### Inputs

| Input | Default | Description |
| ----- | ------- | ----------- |
| `fail-on-severity` | _(unset)_ | If set to one of `low`/`medium`/`high`/`critical`, the action exits 1 when any finding at that level or above is present. If unset, the action always exits 0. |
| `anchor-sentinel-version` | `0.3.0` | Which version of the `anchor-sentinel` crate to install. Pin to a specific version for reproducibility. |
| `working-directory` | `.` | The directory to scan. The default assumes the Anchor project is at the repo root. |
| `extra-args` | _(empty)_ | Extra arguments to pass to `sentinel scan`. For example, `--ignore missing_mut`. |

### Full example

```yaml
name: Security Scan
on:
  push:
    branches: [main]
  pull_request:
permissions:
  contents: read
  security-events: write
jobs:
  sentinel:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Build the IDL — Sentinel needs target/idl/*.json to exist.
      - name: Build Anchor program
        run: anchor build

      - uses: Eniyanyosuva/anchor-sentinel/.github/actions/scan@main
        with:
          anchor-sentinel-version: 0.3.0
          fail-on-severity: high
          extra-args: "--ignore unsafe_arithmetic"
```

## Raw CLI (any CI)

If you can't use the vendored Action (e.g. you're on GitLab,
CircleCI, Buildkite, or a self-hosted runner without network access
to the Actions marketplace), call the CLI directly.

### GitLab CI

```yaml
sentinel_scan:
  image: rust:1-bookworm
  before_script:
    - cargo install anchor-sentinel --locked
    - apt-get update && apt-get install -y libssl-dev libudev-dev
  script:
    - anchor build
    - sentinel scan . --format sarif > sentinel.sarif
    - sentinel scan . --strict --min-severity high
  artifacts:
    reports:
      sast: sentinel.sarif
    when: always
```

### CircleCI

```yaml
version: 2.1
jobs:
  sentinel:
    docker:
      - image: rust:1-bookworm
    steps:
      - checkout
      - run: cargo install anchor-sentinel --locked
      - run: anchor build
      - run: sentinel scan . --format sarif > sentinel.sarif
      - store_artifacts:
          path: sentinel.sarif
      - run: |
          if ! sentinel scan . --strict --min-severity high; then
            echo "Sentinel found high/critical findings."
            exit 1
          fi
```

## Required permissions

For the GitHub Action to upload SARIF to Code Scanning, the
workflow must declare:

```yaml
permissions:
  contents: read        # to checkout the repo
  security-events: write   # to upload SARIF
```

## Required setup

The `anchor build` step must run **before** the Sentinel step.
Sentinel needs `target/idl/<program>.json` to exist. Without
IDL files, Sentinel prints:

```
error: no IDL files found. Run `anchor build` inside the project first
so that target/idl/*.json is populated.
```

…and exits 2. This is the most common reason CI fails on the
Sentinel step. Make sure `anchor build` ran successfully and
`target/idl/*.json` files are checked in or built fresh in the
workflow.

## Triage workflow

When a finding is reported on a PR:

1. **Read the message.** The rule's intent is in plain English,
   with a fix suggestion.
2. **Read the hint.** The hint is the rule's recommended fix —
   e.g. "Type the field as `Signer` instead of `AccountInfo`."
3. **Confirm the bug.** Sentinel flags *patterns*; you still need
   to verify the pattern actually applies to your code.
4. **Fix or suppress.** Either fix the source, or add
   `--ignore <rule>` if the rule is firing on a known-safe
   pattern in your codebase.

## Suppressing false positives

For project-wide suppression, add a `.sentinel-ignore` file at
the repo root (not yet implemented — see [open
issues](https://github.com/Eniyanyosuva/anchor-sentinel/issues))
or pass `--ignore` flags.

For per-PR suppression, comment on the finding and triage it via
the **Code Scanning** UI. Suppression-state persists across
runs.
