# Allium Code Quality Guidelines

The quality gate is intentionally small and high-signal. Run it locally with:

```sh
just verify
```

## Checks

| Tool | Purpose |
|---|---|
| cargo fmt | Enforce consistent code formatting |
| cargo clippy | Detect errors, risky patterns, and enforce idiomatic Rust |
| cargo test | Run unit and integration tests |

## Local hook

`just setup` installs `scripts/pre_push_hook.sh` into `.git/hooks/pre-push`. Every push runs `just verify`.

## Continuous integration

GitHub Actions runs `just verify` for every push to `main`.
