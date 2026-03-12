import React from "react";

const ASCII_LOGO = [
    "┏┓  ┓  ┓ ┏ ",
    "┃┃┏┓┃┓┏┃┃┃┏",
    "┣┛┗┛┗┗┫┗┻┛┛",
    "      ┛    ",
].join("\n");

export default function TuiSection() {
    return (
        <section id="tui" className="py-24 md:py-32 bg-[#030304] border-y border-white/5 relative overflow-hidden">
            <div className="absolute left-0 top-0 w-1/3 h-[400px] bg-primary opacity-5 blur-[120px] pointer-events-none" />
            <div className="absolute right-0 bottom-0 w-1/3 h-[400px] bg-tertiary opacity-5 blur-[120px] pointer-events-none" />

            <div className="text-center mb-12 max-w-4xl mx-auto px-4 relative z-10">
                <div className="font-mono text-primary text-xs uppercase tracking-widest mb-3">INTERACTIVE DASHBOARD</div>
                <h2 className="text-4xl md:text-5xl font-heading font-bold tracking-tight mb-4">
                    The Operations <span className="text-gradient-primary">Dashboard</span>
                </h2>
                <p className="mt-4 text-lg text-muted max-w-2xl mx-auto font-body">
                    Run{" "}<code className="bg-white/10 px-1.5 py-0.5 font-mono text-primary">polyws</code>{" "}with no arguments to enter the keyboard-driven terminal UI.
                </p>
            </div>

            {/* Double-border screen frame */}
            <div className="max-w-5xl mx-auto px-4 relative z-10 mb-16">
                {/* Outer border — cyan tech glow */}
                <div className="p-[3px] rounded-xl" style={{ border: "2px solid rgba(0,210,255,0.45)", boxShadow: "0 0 32px rgba(0,210,255,0.22), 0 0 80px rgba(0,210,255,0.1), inset 0 0 24px rgba(0,0,0,0.6)" }}>
                    {/* Inner border — second ring */}
                    <div className="rounded-lg overflow-hidden" style={{ border: "1px solid rgba(0,210,255,0.2)", background: "#030304" }}>

                        {/* ASCII Logo Banner */}
                        <div className="flex justify-center py-6 border-b overflow-x-auto" style={{ borderColor: "rgba(255,255,255,0.08)", background: "rgba(0,0,0,0.5)" }}>
                            <pre className="font-mono leading-tight whitespace-pre select-none" style={{ fontSize: "clamp(0.5rem, 1.5vw, 1rem)", background: "linear-gradient(90deg, #00D2FF 0%, #3A7BD5 35%, #805AD7 65%, #FF42A1 100%)", WebkitBackgroundClip: "text", WebkitTextFillColor: "transparent", backgroundClip: "text" }}>
                                {ASCII_LOGO}
                            </pre>
                        </div>

                        {/* Title bar: [polyws] [ws] + tabs */}
                        <div className="flex items-center justify-between px-3 py-2 border-b font-mono text-[10px] md:text-[11px] font-bold" style={{ background: "rgba(255,255,255,0.04)", borderColor: "rgba(255,255,255,0.1)" }}>
                            <div className="flex items-center gap-0.5 flex-wrap">
                                <span className="px-1.5 py-0.5 font-bold text-black text-[10px] md:text-[11px] rounded-sm bg-[#00D2FF] leading-none">polyws</span>
                                <span className="text-white/20 mx-1 leading-none">│</span>
                                <span className="font-bold px-0.5 text-[#00D2FF] whitespace-nowrap">1: Projects</span>
                                {["2: Graph", "3: Snapshots", "4: Sync", "5: Logs"].map(t => (
                                    <span key={t} className="text-white/40 hover:text-white cursor-pointer px-0.5 whitespace-nowrap transition">{t}</span>
                                ))}
                            </div>
                            <div className="text-white/30 text-[9px] md:text-[10px] hidden sm:block whitespace-nowrap">polyws v1.0.1</div>
                        </div>

                        {/* Projects table */}
                        <div className="overflow-x-auto">
                            <table className="w-full text-left font-mono text-sm">
                                <thead>
                                    <tr className="text-[10px] uppercase tracking-wider" style={{ borderBottom: "1px solid rgba(255,255,255,0.06)", color: "rgba(255,255,255,0.35)" }}>
                                        <th className="py-3 px-4 font-normal">Repository</th>
                                        <th className="py-3 px-4 font-normal">Branch</th>
                                        <th className="py-3 px-4 font-normal">Status</th>
                                        <th className="py-3 px-4 font-normal">Depends On</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr style={{ background: "rgba(0,210,255,0.07)", borderBottom: "1px solid rgba(255,255,255,0.04)" }}>
                                        <td className="py-3 px-4 font-bold text-[#00D2FF]">core</td>
                                        <td className="py-3 px-4 text-white">main</td>
                                        <td className="py-3 px-4 text-[#00D2FF]">✔ clean</td>
                                        <td className="py-3 px-4 text-white/30">—</td>
                                    </tr>
                                    <tr style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }} className="hover:bg-white/[0.02]">
                                        <td className="py-3 px-4 text-white/80">plugins</td>
                                        <td className="py-3 px-4 text-white">main</td>
                                        <td className="py-3 px-4 text-[#805AD7]">~ modified (3 files)</td>
                                        <td className="py-3 px-4 text-white/40">core</td>
                                    </tr>
                                    <tr style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }} className="hover:bg-white/[0.02]">
                                        <td className="py-3 px-4 text-white/80">infra</td>
                                        <td className="py-3 px-4 text-white">develop</td>
                                        <td className="py-3 px-4 text-[#00D2FF]">✔ clean</td>
                                        <td className="py-3 px-4 text-white/40">core, plugins</td>
                                    </tr>
                                    <tr className="hover:bg-white/[0.02]">
                                        <td className="py-3 px-4 text-white/80">frontend</td>
                                        <td className="py-3 px-4 text-white">feature/ui</td>
                                        <td className="py-3 px-4" style={{ color: "#FF42A1" }}>✘ missing — run pull</td>
                                        <td className="py-3 px-4 text-white/40">infra</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>

                        {/* Status bar */}
                        <div className="flex justify-between items-center px-4 py-3 border-t text-[10px] text-white/40 font-mono uppercase tracking-widest" style={{ borderColor: "rgba(255,255,255,0.1)", background: "rgba(255,255,255,0.015)" }}>
                            <div className="flex gap-4">
                                {[["A","Add"],["D","Delete"],["P","Pull"],["E","Exec"],["R","Refresh"]].map(([k, v]) => (
                                    <span key={k}><span className="text-[#00D2FF] font-bold">{k}</span>: {v}</span>
                                ))}
                            </div>
                            <div>5 projects · 1 snapshot · sync: <span className="text-[#00D2FF] animate-pulse">● running</span></div>
                        </div>

                    </div>
                </div>
            </div>

            {/* Key Bindings */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto px-4 z-10 relative">
                {[
                    { color: "#00D2FF", label: "Projects Tab", keys: [["a","Add new repo via form"],["d","Remove selected repo"],["p","Pull selected repo"],["e","Exec command in all repos"],["r","Refresh git statuses"]] },
                    { color: "#805AD7", label: "Snapshots Tab", keys: [["c","Create new snapshot"],["r","Restore selected snapshot"],["↑↓","Navigate snapshot list"]] },
                    { color: "#FF42A1", label: "Global Keys", keys: [["1-5","Jump to tab directly"],["Tab","Next tab"],["q","Quit TUI"]] },
                ].map(({ color, label, keys }) => (
                    <div key={label} className="bg-surface p-6 rounded-2xl" style={{ border: `1px solid ${color}33` }}>
                        <div className="font-mono text-sm uppercase tracking-widest mb-4 flex items-center gap-2 pb-3" style={{ color, borderBottom: `1px solid ${color}22` }}>
                            <div className="w-1.5 h-1.5 rounded-full" style={{ background: color }} />
                            {label}
                        </div>
                        <ul className="space-y-3 font-mono text-xs text-white/40">
                            {keys.map(([k, v]) => (
                                <li key={k} className="flex gap-3">
                                    <span className="w-8 font-bold" style={{ color }}>{k}</span>
                                    <span>{v}</span>
                                </li>
                            ))}
                        </ul>
                    </div>
                ))}
            </div>
        </section>
    );
}
