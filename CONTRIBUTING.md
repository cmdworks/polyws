# Contributing to polyws

Contributions are welcome: bug fixes, UX improvements, docs updates, and new features.

## 1. Fork and branch

```bash
git checkout -b feat/short-description
```

## 2. Make changes

- Keep changes focused.
- Update docs when behavior changes.
- Add tests for new logic when possible.

## 3. Run checks locally

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

For website changes:

```bash
cd website
npm run lint
npm run build
```

## 4. Commit style

Use clear commit messages, for example:

- `feat: add push command for single project`
- `fix: avoid tui tab overflow on narrow terminals`
- `docs: add config compatibility section`

## 5. Open pull request

Include:

- What changed.
- Why it changed.
- How it was tested.
- Screenshots/GIFs for TUI or website UI changes.

## 6. Review expectations

- CI must pass.
- Keep backward compatibility unless change is intentional and documented.
- Address review comments with follow-up commits.
