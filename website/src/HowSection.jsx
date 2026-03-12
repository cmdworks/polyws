import React from "react";

const steps = [
    {
        num: "01",
        cmd: "polyws init",
        desc: "Creates a workspace config in the current directory. Your workspace root is established and all polyws commands resolve relative to it.",
    },
    {
        num: "02",
        cmd: "polyws add core git@github.com:org/core.git --sync-url git@gitlab.com:backup/core.git",
        desc: "Declare each repo. Add optional --branch, --depends-on, and --sync-url flags. The dependency graph builds itself.",
    },
    {
        num: "03",
        cmd: "polyws bootstrap",
        desc: "Runs doctor to validate the environment, then clones every repo in dependency order. One command, fresh machine ready.",
    },
    {
        num: "04",
        cmd: "polyws",
        desc: "Opens the interactive TUI. Browse status, add repos, pull, exec commands, manage snapshots—all from a precise, keyboard-driven interface.",
    },
];

export default function HowSection() {
    return (
        <section id="how" className="py-24 md:py-32 bg-background relative overflow-hidden">
            {/* Ambient Lighting */}
            <div className="absolute top-0 right-1/4 w-[400px] h-[400px] bg-primary opacity-5 blur-[120px] rounded-full pointer-events-none -z-10">.</div>

            <div className="text-center mb-20 max-w-4xl mx-auto px-4 relative z-10">
                <div className="font-mono text-tertiary text-xs uppercase tracking-widest mb-3 flex items-center justify-center gap-2">
                    <span className="w-4 h-px bg-tertiary"></span>
                    Workflow Protocol
                    <span className="w-4 h-px bg-tertiary"></span>
                </div>
                <h2 className="text-4xl md:text-5xl lg:text-6xl font-heading font-bold tracking-tight mb-4">
                    From Zero to Workspace in<br />
                    <span className="text-gradient-primary">Four Commands</span>
                </h2>
            </div>

            <div className="max-w-4xl mx-auto p-4 relative z-10">
                {/* Blockchain Timeline Vertical Line */}
                <div className="absolute left-[36px] top-8 bottom-8 w-px bg-gradient-to-b from-primary via-primary/20 to-transparent hidden md:block"></div>

                <div className="flex flex-col gap-12 md:gap-16">
                    {steps.map((step, i) => (
                        <div key={i} className="group relative flex flex-col md:flex-row items-start md:items-center gap-6 md:gap-12 pl-4 md:pl-0">

                            {/* Mobile line (hidden on desktop) */}
                            <div className="absolute left-[24px] top-[40px] bottom-[-48px] w-px bg-primary/20 md:hidden last:hidden"></div>

                            {/* Node Number */}
                            <div className="relative shrink-0 w-10 h-10 md:w-20 md:h-20 bg-surface border border-primary/40 rounded-full flex items-center justify-center z-10 font-mono text-base md:text-xl text-primary shadow-[0_0_15px_rgba(0,210,255,0.2)] group-hover:shadow-[0_0_25px_rgba(0,210,255,0.6)] group-hover:scale-110 group-hover:bg-primary/10 transition-all duration-300">
                                {step.num}
                                {/* Pulse Ring */}
                                <div className="absolute inset-0 rounded-full border border-primary/0 group-hover:border-primary/50 group-hover:animate-ping-slow transition-all"></div>
                            </div>

                            {/* Card Content */}
                            <div className="flex-1 bg-surface/80 backdrop-blur-sm border border-white/10 rounded-2xl p-6 md:p-8 hover:border-primary/30 hover:-translate-y-1 transition-all duration-300">
                                <div className="font-mono text-white text-base md:text-lg mb-3 bg-black/50 px-4 py-2 rounded-lg border border-white/5 inline-block w-full overflow-x-auto whitespace-nowrap">
                                    <span className="text-primary mr-2">&gt;</span>{step.cmd}
                                </div>
                                <p className="text-muted text-lg leading-relaxed font-body">{step.desc}</p>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </section>
    );
}
