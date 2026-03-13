import React from "react";
import Footer from "../Footer";
import Navbar from "../Navbar";

const ENTRIES = [
  {
    version: "1.0.3",
    date: "2026-03-13",
    items: [
      "TUI Projects pane: new (i) key re-inits .git and restores remote when .git is deleted locally.",
      "TUI Projects pane: (F) force-push with optional commit prompt; (l) flush — auto-commit all changes with timestamp and force-push.",
      "Sync / push_sync_branch now uses an ISO-8601 timestamp as the commit message (e.g. 'polyws sync 2026-03-13T15:30:00') so every sync is a recoverable snapshot.",
      "Status column shows orange 'no .git — press i to restore' when .git folder is missing but project directory exists.",
      "Navbar highlights the active page (docs / changelog / generet) with a bottom border accent.",
    ],
  },
  {
    version: "1.0.2",
    date: "2026-03-12",
    items: [
      "TUI pull runs in the background with live log streaming.",
      "Workspace root repo supported via path '.' with safe in-place init.",
      "Generator: planner-only workflow with live tree and config output.",
      "Sync daemon now uses a dedicated sync branch with auto-commit and serialized git operations.",
    ],
  },
  {
    version: "1.0.1",
    date: "2026-03-12",
    items: [
      "Config supports JSON and TOML across .poly/.polyws file names.",
      "Config generator and docs pages added to website.",
    ],
  },
];

export default function ChangelogPage() {
  return (
    <div className="min-h-screen bg-background text-foreground font-mono relative overflow-x-hidden">
      <Navbar />

      <main className="max-w-[1100px] mx-auto px-4 pt-28 pb-8">
        <header className="flex flex-wrap items-center justify-between gap-3 mb-4">
          <div>
            <h1 className="text-2xl md:text-3xl font-bold tracking-wide text-gradient-primary">
              polyws Changelog
            </h1>
            <p className="text-xs text-muted mt-1">
              Release notes and feature updates.
            </p>
          </div>
        </header>

        <div className="space-y-4">
          {ENTRIES.map((entry) => (
            <section
              key={entry.version}
              className="bg-surface/90 border border-white/15 rounded-2xl p-4"
            >
              <div className="flex flex-wrap items-center justify-between gap-2 mb-2">
                <h2 className="text-primary text-sm uppercase tracking-[0.18em]">
                  v{entry.version}
                </h2>
                <span className="text-xs text-muted">{entry.date}</span>
              </div>
              <ul className="text-sm text-muted space-y-1 list-disc list-inside">
                {entry.items.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      </main>

      <Footer />
    </div>
  );
}
