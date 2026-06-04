# CLI reference

Full documentation for every flag and subcommand. The on-disk CLI
is generated from the same source; `sentinel <command> --help`
prints a condensed version of what's below.

---

## `sentinel scan <PATH>`

Scan an Anchor project for security issues.

| Argument | Required | Description |
| -------- | -------- | ----------- |
| `<PATH>` | yes | The Anchor project directory (the one that contains `Anchor.toml`) |

### Flags

| Flag | Description |
| ---- | ----------- |
| `--format <FMT>` | Output format. One of `text` (default, human-readable), `json` (machine-readable, byte-stable), `sarif` (SARIF 2.1.0 for GitHub Code Scanning). |
| `--strict` | Treat any non-info finding as a build failure. Exit code becomes `1`. |
| `--min-severity <SEV>` | Only show findings at or above this severity. One of `info`, `low`, `medium`, `high`, `critical`. Exit code becomes `1` if any matching finding is present. |
| `--ignore <RULE>` | Skip findings from this rule. May be passed multiple times. Rule names are the IDs printed by `sentinel rules`. |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

### Examples

```sh
# Default human-readable scan
sentinel scan .

# JSON output, piped to jq
sentinel scan . --format json | jq '.findings[] | {rule, severity, file, line}'

# SARIF for GitHub Code Scanning
sentinel scan . --format sarif > sentinel.sarif

# CI gate: build red on high or critical
sentinel scan . --strict --min-severity high

# Suppress known false positives
sentinel scan . --ignore missing_mut --ignore unsafe_arithmetic
```

### Exit codes

| Code | Meaning |
| ---- | ------- |
| `0`  | No findings (or findings below the threshold) |
| `1`  | Findings present and `--strict` (or `--min-severity`) triggered |
| `2`  | Tool error — bad path, no IDL files, parse failure |
| `130` | Interrupted (Ctrl+C) |

### Output formats

**`text`** (default) — human-readable, possibly-animated, color on
TTYs. Animations include: a 3-phase stderr spinner
(`⠋ Loading IDL files…` → `⠙ Parsing Rust source…` → `⠹ Running N rules…`),
a 35ms reveal between finding blocks, and a 25ms/char bar-fill
animation in the summary footer. All animations are auto-disabled
when `stdout` is not a TTY, `CI=true`, or `NO_COLOR` is set.

**`json`** — machine-readable, byte-stable, no ANSI codes. Schema:

```json
{
  "findings": [
    {
      "rule": "missing_signer",
      "severity": "critical",
      "program": "vault",
      "instruction": "withdraw",
      "account": "authority",
      "file": "programs/vault/src/lib.rs",
      "line": 42,
      "column": 4,
      "message": "Account `authority` is not marked as a signer...",
      "hint": "Type the field as `Signer` or add `#[account(signer)]`."
    }
  ],
  "summary": {
    "critical": 7,
    "high": 3,
    "medium": 4,
    "low": 0,
    "info": 0,
    "total": 14
  }
}
```

**`sarif`** — SARIF 2.1.0, ready for `actions/upload-sarif` or VS Code.

---

## `sentinel rules`

List all 13 registered rules. Output is a table:

```
⚓ anchor-sentinel — 13 rules active

┌─────┬────────────────────────────────────┬──────────┬───────┐
│   # │ Rule                               │ Severity │ Layer │
├─────┼────────────────────────────────────┼──────────┼───────┤
│   1 │ cpi_signer_seed_validation         │ CRITICAL │ AST   │
│   2 │ missing_balance_check              │ CRITICAL │ AST   │
...
│  13 │ unsafe_arithmetic                  │ MEDIUM   │ AST   │
└─────┴────────────────────────────────────┴──────────┴───────┘
```

- Sorted by severity (Critical first), then alphabetical.
- `Severity` is the rule's default severity (you can still gate
  with `--min-severity`).
- `Layer` is `IDL+AST` (uses both signals), `AST` (no IDL data
  needed), or `IDL` (no source needed). See [rules.md](rules.md)
  for which is which.

---

## `sentinel version`

Print the sentinel version (e.g. `sentinel 0.3.0`).

---

## Environment variables

| Variable | Effect |
| -------- | ------ |
| `NO_COLOR` | When set to any non-empty value, disables all ANSI color and animation. Per [no-color.org](https://no-color.org/). |
| `CI` | When set to `true`, disables animations. Color is unaffected (CI logs may want it). |
| `TERM=dumb` | Same effect as `CI=true`. |

## Common patterns

**Pre-commit hook** — run a quick scan before each commit:

```sh
#!/usr/bin/env bash
# .git/hooks/pre-commit
sentinel scan . --min-severity high || {
  echo "Sentinel found high/critical findings. Commit aborted."
  exit 1
}
```

**Watch mode** — re-run on file changes:

```sh
find programs -name '*.rs' | entr -c sentinel scan . --min-severity medium
```

(Requires [`entr`](https://github.com/eradman/entr).)
