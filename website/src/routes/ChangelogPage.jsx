import React from "react";
import { Link } from "react-router-dom";
import Footer from "../Footer";
import Navbar from "../Navbar";

const ENTRIES = [
  {
    version: "1.0.2",
    date: "2026-03-12",
    items: [
      "TUI pull runs in the background with live log streaming.",
      "Workspace root repo supported via path '.' with safe in-place init.",
      "Generator: planner-only workflow with live tree and config output.",
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
  const base = import.meta.env.BASE_URL;

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
          <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-wider">
            <Link
              to="/docs"
              className="px-3 py-2 border border-primary/30 rounded-lg hover:bg-primary/10 transition"
            >
              Docs
            </Link>
            <a
              href={`${base}changelog`}
              className="px-3 py-2 bg-gradient-primary text-black rounded-lg font-semibold"
            >
              /polyws/changelog
            </a>
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
