# User Guide

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/install.sh | bash
```

## First workspace

```bash
mkdir my-workspace && cd my-workspace
polyws doctor
polyws init
polyws add core git@github.com:org/core.git --path apps/platform/core
polyws add api git@github.com:org/api.git --path services/api --depends-on core
polyws pull
polyws
```

## Config file names

polyws reads JSON or TOML from:

- `.polyws`
- `.poly`
- `.polyws.json`
- `.poly.json`
- `.polyws.toml`
- `.poly.toml`

## Daily commands

- `polyws pull [name]` clone/update repos.
- `polyws push [name]` push one repo or all repos.
- `polyws exec "<cmd>"` run command across repos with dependency order.
- `polyws status` show branch/dirty status.
- `polyws graph` show dependency graph.
- `polyws snapshot create` save multi-repo state.
- `polyws snapshot restore <file>` restore saved state.

## Nested directories

Use `--path` to model monorepo-like trees:

```bash
polyws add plugins git@github.com:org/plugins.git --path apps/platform/plugins
polyws add auth git@github.com:org/auth.git --path services/api/auth --depends-on core
```

Each project path must be unique and relative to workspace root.
