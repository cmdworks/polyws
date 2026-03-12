import React from "react";
import { Link } from "react-router-dom";
import Footer from "../Footer";
import Navbar from "../Navbar";

const FILE_ROWS = [
  [".polyws", "JSON or TOML", "auto-detected by parser"],
  [".poly", "JSON or TOML", "auto-detected by parser"],
  [".polyws.json", "JSON", "explicit extension"],
  [".poly.json", "JSON", "explicit extension"],
  [".polyws.toml", "TOML", "explicit extension"],
  [".poly.toml", "TOML", "explicit extension"],
];

const USER_COMMANDS = [
  ["polyws init", "Create workspace config in current directory"],
  ["polyws add <name> <url>", "Add project with optional --path, --branch, --depends-on"],
  ["polyws pull [name]", "Clone or update all projects, or one project"],
  ["polyws push [name]", "Push one project or all projects to origin"],
  ["polyws exec \"<cmd>\"", "Run command in dependency order in parallel levels"],
  ["polyws snapshot create", "Save current repo heads as snapshot"],
  ["polyws sync start", "Start background mirror daemon for sync_url remotes"],
];

const DEV_NOTES = [
  "Core CLI command definitions live in src/cli.rs and dispatch from src/main.rs.",
  "Workspace orchestration is centered in src/workspace.rs with config loading in src/config.rs.",
  "TUI logic is in src/tui/; keep new rendering code modular by section.",
  "For website updates, keep all route UI under website/src and avoid standalone public HTML pages.",
  "Release binaries are built only on version tags (v*), while website deploys on main when website files change.",
];

const QUICK_START = `polyws doctor
polyws init
polyws add core git@github.com:org/core.git --path apps/platform/core
polyws add plugins git@github.com:org/plugins.git --path apps/platform/plugins --depends-on core
polyws pull
polyws`;

const JSON_EXAMPLE = `{
  "name": "my-workspace",
  "sync_interval_minutes": 30,
  "projects": [
    {
      "name": "core",
      "path": "apps/platform/core",
      "url": "git@github.com:org/core.git",
      "branch": "main"
    },
    {
      "name": "api",
      "path": "services/api",
      "url": "git@github.com:org/api.git",
      "branch": "main",
      "depends_on": ["core"]
    }
  ]
}`;

const TOML_EXAMPLE = `name = "my-workspace"
sync_interval_minutes = 30

[[projects]]
name = "core"
path = "apps/platform/core"
url = "git@github.com:org/core.git"
branch = "main"

[[projects]]
name = "api"
path = "services/api"
url = "git@github.com:org/api.git"
branch = "main"
depends_on = ["core"]`;

export default function DocsPage() {
  const base = import.meta.env.BASE_URL;
  const docsUrl = `${base}docs`;

  return (
    <div className="min-h-screen bg-background text-foreground font-mono relative overflow-x-hidden">
      <Navbar />

      <main className="max-w-[1240px] mx-auto px-4 pt-28 pb-8">
        <header className="flex flex-wrap items-center justify-between gap-3 mb-4">
          <div>
            <h1 className="text-2xl md:text-3xl font-bold tracking-wide text-gradient-primary">
              polyws Docs
            </h1>
            <p className="text-xs text-muted mt-1">
              User docs and developer docs for workspace setup, operations, and contribution flow.
            </p>
          </div>
          <div className="flex flex-wrap gap-2 text-xs uppercase tracking-wider">
            <Link
              to="/generet"
              className="px-3 py-2 border border-primary/30 rounded-lg hover:bg-primary/10 transition"
            >
              Config Generator
            </Link>
            <a
              href={docsUrl}
              className="px-3 py-2 bg-gradient-primary text-black rounded-lg font-semibold"
            >
              /polyws/docs
            </a>
          </div>
        </header>

        <div className="grid grid-cols-1 lg:grid-cols-[240px_1fr] gap-4">
          <aside className="bg-surface/90 border border-white/15 rounded-2xl p-4 h-fit">
            <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-3">On This Page</h2>
            <div className="space-y-2 text-xs">
              <a href="#quickstart" className="block text-muted hover:text-foreground">Quick Start</a>
              <a href="#commands" className="block text-muted hover:text-foreground">Command Reference</a>
              <a href="#config" className="block text-muted hover:text-foreground">Config & Formats</a>
              <a href="#nested" className="block text-muted hover:text-foreground">Nested Paths</a>
              <a href="#dev" className="block text-muted hover:text-foreground">Developer Docs</a>
              <a href="#contrib" className="block text-muted hover:text-foreground">Contributing</a>
            </div>
          </aside>

          <div className="space-y-4">
            <section id="quickstart" className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Quick Start</h2>
              <p className="text-xs text-muted mb-2">
                Recommended first workflow to initialize and pull a multi-repo workspace.
              </p>
              <pre className="bg-black/70 border border-white/10 rounded-xl p-3 overflow-auto text-xs leading-6">
                {QUICK_START}
              </pre>
            </section>

            <section id="commands" className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-3">Command Reference</h2>
              <div className="overflow-auto border border-white/10 rounded-xl">
                <table className="w-full text-xs">
                  <thead className="bg-black/35 text-primary uppercase tracking-wider">
                    <tr>
                      <th className="text-left px-3 py-2">Command</th>
                      <th className="text-left px-3 py-2">Purpose</th>
                    </tr>
                  </thead>
                  <tbody>
                    {USER_COMMANDS.map(([cmd, desc]) => (
                      <tr key={cmd} className="border-t border-white/10">
                        <td className="px-3 py-2 text-cyan-300">{cmd}</td>
                        <td className="px-3 py-2 text-muted">{desc}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>

            <section id="config" className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-3">Config & Formats</h2>
              <p className="text-xs text-muted mb-3">
                polyws accepts both JSON and TOML in multiple file names.
              </p>
              <div className="overflow-auto border border-white/10 rounded-xl mb-4">
                <table className="w-full text-xs">
                  <thead className="bg-black/35 text-primary uppercase tracking-wider">
                    <tr>
                      <th className="text-left px-3 py-2">File Name</th>
                      <th className="text-left px-3 py-2">Format</th>
                      <th className="text-left px-3 py-2">Notes</th>
                    </tr>
                  </thead>
                  <tbody>
                    {FILE_ROWS.map(([name, format, notes]) => (
                      <tr key={name} className="border-t border-white/10">
                        <td className="px-3 py-2 text-cyan-300">{name}</td>
                        <td className="px-3 py-2 text-foreground/90">{format}</td>
                        <td className="px-3 py-2 text-muted">{notes}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
                <pre className="bg-black/70 border border-white/10 rounded-xl p-3 overflow-auto text-xs leading-6">
                  {JSON_EXAMPLE}
                </pre>
                <pre className="bg-black/70 border border-white/10 rounded-xl p-3 overflow-auto text-xs leading-6">
                  {TOML_EXAMPLE}
                </pre>
              </div>
            </section>

            <section id="nested" className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Nested Paths</h2>
              <p className="text-xs text-muted mb-2">
                Use nested paths to map repos into directories/subdirectories.
              </p>
              <pre className="bg-black/70 border border-white/10 rounded-xl p-3 overflow-auto text-xs leading-6">{`polyws add core git@github.com:org/core.git --path apps/platform/core
polyws add api git@github.com:org/api.git --path services/api --depends-on core`}</pre>
            </section>

            <section id="dev" className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Developer Docs</h2>
              <ul className="list-disc list-inside text-xs text-muted space-y-1">
                {DEV_NOTES.map((note) => (
                  <li key={note}>{note}</li>
                ))}
              </ul>
              <p className="text-xs text-muted mt-3">
                Run locally: <code className="text-cyan-300">cargo test --all</code>,{" "}
                <code className="text-cyan-300">cargo clippy -- -D warnings</code>,{" "}
                <code className="text-cyan-300">cargo fmt -- --check</code>,{" "}
                <code className="text-cyan-300">cd website && npm run lint && npm run build</code>.
              </p>
            </section>

            <section id="contrib" className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Contributing</h2>
              <p className="text-xs text-muted">
                Contributions are welcome. Follow the contribution guide for setup, quality checks, and PR flow.
              </p>
              <a
                href="https://github.com/cmdworks/polyws/blob/main/CONTRIBUTING.md"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex mt-3 px-3 py-2 text-xs uppercase tracking-wider border border-white/25 rounded-lg hover:bg-white/5"
              >
                Open CONTRIBUTING.md
              </a>
            </section>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
