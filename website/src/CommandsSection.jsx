import React from "react";

const commands = [
    {
        cmd: "polyws",
        args: "—",
        desc: "Launch the interactive TUI (no subcommand)",
        cat: "TUI",
    },
    {
        cmd: "polyws init",
        args: "—",
        desc: "Initialize a workspace config in the current directory (.polyws by default)",
        cat: "workspace",
    },
    {
        cmd: "polyws add",
        args: "<name> <url> [--branch] [--depends-on] [--sync-url]",
        desc: "Add a new project repository to the workspace config",
        cat: "workspace",
    },
    {
        cmd: "polyws remove",
        args: "<name>",
        desc: "Remove a project from the workspace config",
        cat: "workspace",
    },
    {
        cmd: "polyws list",
        args: "—",
        desc: "Print a formatted table of all projects: name, branch, dependencies",
        cat: "workspace",
    },
    {
        cmd: "polyws status",
        args: "—",
        desc: "Show git status summary for every repo",
        cat: "git",
    },
    {
        cmd: "polyws pull",
        args: "[name] [--force]",
        desc: "Clone missing repos and safe-pull existing ones. Use --force to allow hard reset when needed.",
        cat: "git",
    },
    {
        cmd: "polyws clone",
        args: "[name] [--force]",
        desc: "Alias for pull with the exact same behavior and flags.",
        cat: "git",
    },
    {
        cmd: "polyws push",
        args: "[name]",
        desc: "Push local branch to origin for one repo or all repos (dependency-level parallel).",
        cat: "git",
    },
    {
        cmd: "polyws graph",
        args: "—",
        desc: "Render the dependency graph as an ASCII tree",
        cat: "workspace",
    },
    {
        cmd: "polyws exec",
        args: "<cmd>",
        desc: "Run a shell cmd in every repo in parallel, respecting dependencies",
        cat: "exec",
    },
    {
        cmd: "polyws snapshot create",
        args: "—",
        desc: "Capture current commit hashes to .polyws/snapshots/",
        cat: "snapshot",
    },
    {
        cmd: "polyws snapshot restore",
        args: "<file> [--dry-run] [-y]",
        desc: "Preview or restore a saved snapshot with confirmation controls.",
        cat: "snapshot",
    },
    {
        cmd: "polyws doctor",
        args: "—",
        desc: "Validate the full environment: git, ssh, network, disk, repos",
        cat: "health",
    },
    {
        cmd: "polyws bootstrap",
        args: "—",
        desc: "Quick-start: runs doctor then pull all. Ideal for fresh machine setup.",
        cat: "health",
    },
    {
        cmd: "polyws sync start",
        args: "—",
        desc: "Start the background mirror-sync daemon (PID tracked)",
        cat: "sync",
    },
    {
        cmd: "polyws vm setup",
        args: "—",
        desc: "Bootstrap the remote VM via SSH",
        cat: "vm",
    },
];

const catColors = {
    TUI: "text-tertiary border-tertiary",
    workspace: "text-primary border-primary",
    git: "text-white border-white/50",
    exec: "text-secondary border-secondary",
    snapshot: "text-white border-white/50",
    health: "text-primary border-primary",
    sync: "text-tertiary border-tertiary",
    vm: "text-secondary border-secondary",
};

export default function CommandsSection() {
    return (
        <section id="commands" className="py-24 md:py-32 bg-surface border-y border-white/5 relative">
            <div className="absolute right-0 top-0 w-1/3 h-full bg-grid-pattern opacity-20 pointer-events-none mix-blend-screen"></div>

            <div className="text-left mb-16 max-w-5xl mx-auto px-4 relative z-10">
                <div className="font-mono text-tertiary text-xs uppercase tracking-widest mb-3 flex items-center gap-2">
                    <span className="w-4 h-px bg-tertiary"></span>
                    CLI Reference
                </div>
                <h2 className="text-4xl md:text-5xl font-heading font-bold tracking-tight mb-4">
                    Command Syntax
                </h2>
                <p className="mt-4 text-base md:text-lg text-muted max-w-2xl leading-relaxed font-body">
                    Precise, composable commands that run across the entire dependency graph.
                </p>
            </div>
            
            <div className="max-w-5xl mx-auto px-4 relative z-10">
                <div className="bg-[#030304] border border-white/10 rounded-xl overflow-hidden shadow-2xl">
                    <div className="overflow-x-auto">
                        <table className="w-full text-left border-collapse">
                            <thead>
                                <tr className="border-b border-white/10 bg-white/5 font-mono text-xs text-muted uppercase tracking-widest">
                                    <th className="py-4 px-6 font-medium">Command</th>
                                    <th className="py-4 px-6 font-medium">Args / Flags</th>
                                    <th className="py-4 px-6 font-medium">Description</th>
                                    <th className="py-4 px-6 font-medium whitespace-nowrap">Category</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-white/5 font-mono text-sm">
                                {commands.map((row, i) => (
                                    <tr key={i} className="hover:bg-white/[0.02] transition-colors group">
                                        <td className="py-4 px-6 text-foreground font-medium whitespace-nowrap group-hover:text-primary transition-colors">
                                            {row.cmd}
                                        </td>
                                        <td className="py-4 px-6 text-muted whitespace-nowrap">
                                            {row.args}
                                        </td>
                                        <td className="py-4 px-6 text-muted font-body leading-relaxed max-w-md">
                                            {row.desc}
                                        </td>
                                        <td className="py-4 px-6 whitespace-nowrap">
                                            <span className={`inline-block px-2.5 py-1 text-xs tracking-wider uppercase bg-transparent border rounded-md ${catColors[row.cat] || "text-muted border-white/20"}`}>
                                                {row.cat}
                                            </span>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </section>
    );
}
