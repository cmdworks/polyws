import React from "react";

const AppleIcon = (props) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M16.6 14.1c-.1-2.9 2.4-4.3 2.5-4.4-1.3-1.9-3.4-2.2-4.1-2.2-1.8-.2-3.5 1-4.4 1-.9 0-2.3-1-3.8-1-1.9.1-3.6 1.1-4.6 2.8-2 3.5-.5 8.7 1.4 11.5 1 1.4 2.1 3 3.6 2.9 1.4-.1 2-1 3.7-1 1.7 0 2.2 1 3.7 1 1.6 0 2.5-1.5 3.5-2.9 1.1-1.6 1.6-3.2 1.6-3.3-.1 0-3.3-1.2-3.4-4.4zm-3-8.8c.8-1 1.3-2.3 1.1-3.7-1.2.1-2.7.8-3.5 1.8-.7.8-1.3 2.2-1.1 3.6 1.3.1 2.7-.6 3.5-1.7z" />
    </svg>
);

const LinuxIcon = (props) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M12 2C9.5 2 7 3.5 6 6s-.5 5 .5 7c1 2 2 4 1 6-1 2-3 2-3 2s4 1 8 1 8-1 8-1-2-0-3-2c-1-2 0-4 1-6 1-2 1.5-4.5.5-7-1-2.5-3.5-4-6-4z" />
        <circle cx="10" cy="9" r="1.5" fill="#000" />
        <circle cx="14" cy="9" r="1.5" fill="#000" />
        <path d="M11 12h2a1 1 0 0 1 0 2h-2a1 1 0 0 1 0-2z" fill="#FFA500" />
    </svg>
);

const WindowsIcon = (props) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M2.5 11.5v-6l8-1.1v7.1h-8zm8.5-7.3l9.5-1.4v8.7h-9.5v-7.3zm-8.5 8h8v7.2l-8-1.1v-6.1zm8.5 0h9.5v8.9l-9.5-1.4v-7.5z" />
    </svg>
);

const installCmds = [
    {
        label: "One-line Install (Auto OS & Arch)",
        cmd: "curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/install.sh | bash",
    },
    {
        label: "Install Custom Name (Interactive)",
        cmd: "curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/install.sh | bash",
    },
    {
        label: "Install Custom Name (Non-Interactive)",
        cmd: "polyws_NAME=poly curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/install.sh | bash",
    },
    {
        label: "Specify a Version",
        cmd: "polyws_VERSION=v1.0.0 curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/install.sh | bash",
    },
    {
        label: "Build from Source",
        cmd: [
            "git clone https://github.com/cmdworks/polyws",
            "cd polyws && cargo build --release",
            "cp target/release/polyws /usr/local/bin/polyws",
        ],
    },
];

const quickStartCmds = [
    "mkdir my-workspace && cd my-workspace",
    "polyws init",
    "polyws add core git@github.com:org/core.git --sync-url git@gitlab.com:backup/core.git",
    "polyws add plugins git@github.com:org/plugins.git --depends-on core --sync-url git@gitlab.com:backup/plugins.git",
    "polyws pull --force   # optional: hard reset when needed",
    "polyws push plugins",
    "polyws bootstrap",
    "polyws   # open TUI",
];

const requirements = [
    "curl + bash — pre-built binaries, no Rust needed",
    "git 2.x — required for clone/pull/push operations",
    "ssh — for VM integration and SSH remote URLs",
    "Rust 1.75+ — only if building from source",
];

const platforms = [
    { icon: AppleIcon, label: "macOS arm64 (Apple Silicon)" },
    { icon: LinuxIcon, label: "Linux x86_64" },
    { icon: LinuxIcon, label: "aarch64" },
    { icon: WindowsIcon, label: "Windows x86_64 (Native Executable)" },
    { icon: WindowsIcon, label: "Windows aarch64 (Native Executable)" },
];

export default function InstallSection() {
    return (
        <section id="install" className="py-24 md:py-32 bg-background relative">
            <div className="absolute top-0 right-0 w-full h-[500px] bg-grid-pattern opacity-10 pointer-events-none mix-blend-screen mask-image-top"></div>

            <div className="text-center mb-16 max-w-4xl mx-auto px-4 relative z-10">
                <div className="font-mono text-tertiary text-xs uppercase tracking-widest mb-3 flex items-center justify-center gap-2">
                    <span className="w-4 h-px bg-tertiary"></span>
                    Installation
                    <span className="w-4 h-px bg-tertiary"></span>
                </div>
                <h2 className="text-4xl md:text-5xl font-heading font-bold tracking-tight mb-4">
                    Get Started in <span className="text-gradient-accent">60 Seconds</span>
                </h2>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 lg:gap-12 max-w-6xl mx-auto px-4 relative z-10">
                {/* Code Steps */}
                <div className="flex flex-col gap-6">
                    <div className="flex flex-col gap-4">
                        {installCmds.map((item, i) => (
                            <div key={i} className="group relative bg-[#030304] border border-white/10 rounded-xl overflow-hidden hover:border-primary/40 hover:shadow-[0_0_30px_rgba(0,210,255,0.1)] transition-all duration-300">
                                <div className="flex items-center justify-between px-4 py-2 bg-white/5 border-b border-white/10">
                                    <span className="font-mono text-xs text-muted uppercase tracking-widest">{item.label}</span>
                                    <button
                                        onClick={() => {
                                            const textToCopy = Array.isArray(item.cmd) ? item.cmd.join('\\n') : item.cmd;
                                            navigator.clipboard.writeText(textToCopy);
                                        }}
                                        className="font-mono text-[10px] text-muted hover:text-primary transition uppercase tracking-widest px-2 py-1 bg-white/5 rounded border border-white/10 opacity-0 group-hover:opacity-100"
                                    >
                                        Copy
                                    </button>
                                </div>
                                <div className="p-4 overflow-x-auto">
                                    <pre className="text-sm leading-relaxed font-mono text-foreground">
                                        {Array.isArray(item.cmd)
                                            ? item.cmd.map((c, j) => <div key={j} className="mb-1"><span className="text-primary select-none mr-2">❯</span>{c}</div>)
                                            : <div className="mb-1"><span className="text-secondary select-none mr-2">❯</span>{item.cmd}</div>}
                                    </pre>
                                </div>
                            </div>
                        ))}
                    </div>

                    <div className="group relative bg-[#030304] border border-white/10 rounded-xl overflow-hidden hover:border-tertiary/40 hover:shadow-[0_0_30px_rgba(255,66,161,0.1)] transition-all duration-300">
                        <div className="flex items-center justify-between px-4 py-2 bg-white/5 border-b border-white/10">
                            <span className="font-mono text-xs text-muted uppercase tracking-widest">Quick Start Workspace</span>
                            <button className="font-mono text-[10px] text-muted hover:text-tertiary transition uppercase tracking-widest px-2 py-1 bg-white/5 rounded border border-white/10 opacity-0 group-hover:opacity-100">Copy</button>
                        </div>
                        <div className="p-5 overflow-x-auto">
                            <pre className="text-sm leading-relaxed font-mono text-foreground">
                                <div className="text-muted/60 select-none mb-1"># Create workspace</div>
                                {quickStartCmds.map((c, i) => (
                                    <div key={i} className="mb-1"><span className="text-tertiary select-none mr-2">❯</span>{c}</div>
                                ))}
                            </pre>
                        </div>
                    </div>
                </div>

                {/* Requirements & Platforms */}
                <div className="flex flex-col gap-6">
                    <div className="bg-surface/50 border border-white/10 rounded-xl p-6">
                        <div className="font-mono text-primary text-sm uppercase tracking-widest mb-4 flex items-center gap-2">
                            <div className="w-2 h-2 rounded-full bg-primary animate-pulse"></div>
                            System Requirements
                        </div>
                        <ul className="space-y-3 font-mono text-sm text-muted">
                            {requirements.map((r, i) => (
                                <li key={i} className="flex items-start gap-3">
                                    <span className="text-primary mt-1">◈</span>
                                    <span>{r}</span>
                                </li>
                            ))}
                        </ul>
                    </div>

                    <div className="bg-surface/50 border border-white/10 rounded-xl p-6">
                        <div className="font-mono text-tertiary text-sm uppercase tracking-widest mb-4 flex items-center gap-2">
                            <div className="w-2 h-2 rounded-full bg-tertiary"></div>
                            Platform Support (CI-Built)
                        </div>
                        <div className="grid grid-cols-1 gap-3 font-mono text-sm text-muted">
                            {platforms.map((p, i) => {
                                const Icon = p.icon;
                                return (
                                    <div key={i} className="flex items-center gap-3 bg-[#030304] px-4 py-3 rounded-lg border border-white/5 group hover:border-primary/30 transition-colors">
                                        <div className="flex items-center justify-center w-8 h-8 rounded bg-primary/10 text-primary group-hover:scale-110 transition-transform">
                                            <Icon className="w-5 h-5" />
                                        </div>
                                        <span className="text-foreground">{p.label}</span>
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
}
