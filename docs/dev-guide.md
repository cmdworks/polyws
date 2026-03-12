# Developer Guide

## Local setup

```bash
git clone git@github.com:cmdworks/polyws.git
cd polyws
cargo build
cargo test --all
```

Website:

```bash
cd website
npm install
npm run dev
```

## Code map

- `src/main.rs`: command dispatch.
- `src/cli.rs`: clap command definitions.
- `src/config.rs`: config parsing + dependency graph.
- `src/workspace.rs`: workspace operations (`init/add/remove/pull/push/status`).
- `src/git.rs`: git actions.
- `src/tui/`: interactive terminal UI.
- `src/vm/`: VM setup/sync/exec support.

## Quality checks

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Website checks:

```bash
cd website
npm run lint
npm run build
```

## Release and CI behavior

- Website deploy triggers on `main` pushes that touch `website/**`.
- Binary release builds trigger only on tags matching `v*`.
- Keep feature work on branches and open PRs into `main`.

## Improving the TUI

- Keep rendering and event logic in smaller modules inside `src/tui/`.
- Validate terminal width/height and avoid clipping in narrow terminals.
- Prefer compact labels in nav/help rows.
- Add tests for parsing/state transitions where possible.
