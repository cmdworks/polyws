import React from "react";
import { Network, Cpu, History, GitMerge, RefreshCw, Terminal, Wrench, Server } from "lucide-react";

const features = [
    {
        icon: Network,
        cmd: "polyws init · add · list",
        title: "Workspace Management",
        desc: "Declare your entire repo topology in a single .polyws file. Add, remove, and inspect repos with surgical precision. Dependency ordering is computed automatically from the graph.",
    },
    {
        icon: Cpu,
        cmd: "polyws exec \"…\"",
        title: "Parallel Execution",
        desc: "Run any shell command across every repo simultaneously using high-performance thread pools. Dependency levels are strictly respected—dependents run only after upstreams complete.",
    },
    {
        icon: History,
        cmd: "polyws snapshot list",
        title: "Snapshot & Restore",
        desc: "Capture the exact commit hash of every repo at any moment. Restore the full workspace to that state in seconds. Ship it. Roll it back. Never lose a working state again.",
    },
    {
        icon: GitMerge,
        cmd: "polyws graph",
        title: "Dependency Graph",
        desc: "Visualize your entire repo topology as an ASCII dependency tree. Understand what depends on what, detect cyclic dependencies, and plan your release order intelligently.",
    },
    {
        icon: RefreshCw,
        cmd: "polyws sync start",
        title: "Mirror Sync Daemon",
        desc: "A background daemon that continuously pushes to every configured sync URL. Per-repo intervals, PID tracking, start/stop anywhere—your backup runs transparently.",
    },
    {
        icon: Terminal,
        cmd: "polyws (interactive)",
        title: "Interactive TUI",
        desc: "Jump into a full terminal UI by typing polyws with no args. Browse repos, pull, exec, manage snapshots, and run doctor checks through a keyboard-driven interface.",
    },
    {
        icon: Wrench,
        cmd: "polyws doctor",
        title: "Doctor & Repair",
        desc: "Validate git, ssh, compilers, network, and disk in one pass. Auto-repair broken remotes and re-clone missing repos. Bootstrap a fresh machine instantly.",
    },
    {
        icon: Server,
        cmd: "polyws vm setup",
        title: "VM Integration",
        desc: "SSH into a remote dev VM, bootstrap it, sync your workspace via high-speed protocols, run distant commands, open interactive shells—all through the same unified interface.",
    },
];

export default function FeaturesSection() {
    return (
        <section id="features" className="pt-8 pb-24 md:pt-12 md:pb-32 relative bg-background">
            
            {/* Subtle Texture Overlay */}
            <div className="absolute inset-0 bg-grid-pattern opacity-30 pointer-events-none"></div>

            <div className="text-left mb-16 max-w-7xl mx-auto px-4 relative z-10">
                <div className="font-mono text-primary text-xs uppercase tracking-widest mb-3 flex items-center gap-2">
                    <span className="w-4 h-px bg-primary"></span>
                    Protocol Architecture
                </div>
                <h2 className="text-4xl md:text-5xl lg:text-6xl font-heading font-bold tracking-tight mb-4">
                    Everything Your <br/>
                    <span className="text-gradient-primary">Polyrepo Needs</span>
                </h2>
                <p className="mt-4 text-lg text-muted max-w-2xl leading-relaxed font-body">
                    Engineered for massive scale and absolute reliability. Polyws treats your codebase like a high-frequency trading platform—fast, precise, and uncompromising.
                </p>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 max-w-7xl mx-auto px-4 relative z-10">
                {features.map((f, i) => (
                    <div key={i} className="group relative bg-surface border border-white/10 rounded-2xl p-8 flex flex-col items-start hover:-translate-y-1 hover:border-primary/50 hover:shadow-card-hover transition-all duration-300 overflow-hidden">
                        
                        {/* Huge Watermark Icon */}
                        <f.icon className="absolute -bottom-4 -right-4 w-32 h-32 text-white opacity-5 group-hover:text-primary group-hover:opacity-10 transition-all duration-500 rotate-[-15deg]" />

                        <div className="bg-primary/10 border border-primary/20 rounded-xl p-3 mb-6 group-hover:shadow-[0_0_20px_rgba(0,210,255,0.3)] transition-shadow">
                            <f.icon className="w-6 h-6 text-primary" />
                        </div>
                        
                        <h3 className="font-heading font-bold text-xl tracking-tight mb-2 text-foreground">{f.title}</h3>
                        
                        <div className="font-mono text-xs text-primary/80 mb-4 bg-black/40 px-2.5 py-1 rounded inline-block border border-white/5">
                            &gt; {f.cmd}
                        </div>
                        
                        <p className="text-muted text-sm leading-relaxed relative z-10">
                            {f.desc}
                        </p>
                    </div>
                ))}
            </div>
        </section>
    );
}
