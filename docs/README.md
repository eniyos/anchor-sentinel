# Anchor Sentinel documentation

This directory holds the user-facing reference docs that supplement
the project [README](../README.md). The README is the entry
point — start there, then come here for details.

| Doc | What's in it |
| --- | ------------ |
| [cli.md](cli.md) | Full flag and subcommand reference, exit codes, environment variables, common patterns. |
| [rules.md](rules.md) | All 13 rules: what each one checks, its layer, false-positive notes. |
| [ci.md](ci.md) | GitHub Action reference, raw-CLI recipes for other CI systems, triage workflow. |

## Doc conventions

- Code blocks without a language tag (` ``` `) are shell. Code
  blocks tagged ` ` ```sh ` are shell, ` ` ```yaml ` is YAML,
  ` ` ```text ` is plain text (often an example of CLI output).
- "Severity" is one of `critical`, `high`, `medium`, `low`, `info`,
  in that order.
- "Layer" is one of `IDL`, `AST`, or `IDL+AST`. See [rules.md](rules.md)
  for what each layer means.
- "Triage" means: read the message, read the hint, verify the
  bug applies, fix or suppress.

## Contributing to the docs

The docs live in this directory and are published from the `main`
branch on GitHub. To change them:

```sh
# edit the file
$EDITOR docs/cli.md

# commit and push
git add docs/
git commit -m "docs: clarify --ignore flag"
git push
```

There's no separate build step — Markdown renders directly on
GitHub. Keep formatting simple (no HTML, no embedded images) so
the docs work in any Markdown viewer.
