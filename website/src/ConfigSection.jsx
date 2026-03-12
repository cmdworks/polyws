import React from "react";

export default function ConfigSection() {
    const generatorUrl = `${import.meta.env.BASE_URL}generet`;
    const docsUrl = `${import.meta.env.BASE_URL}docs`;

    return (
        <section id="config" className="py-24 md:py-32 relative bg-surface">
            <div className="text-center mb-16 max-w-4xl mx-auto px-4 z-10 relative">
                <div className="font-mono text-tertiary text-xs uppercase tracking-widest mb-3 flex items-center justify-center gap-2">
                    <span className="w-4 h-px bg-tertiary"></span>
                    Configuration
                    <span className="w-4 h-px bg-tertiary"></span>
                </div>
                <h2 className="text-4xl md:text-5xl font-heading font-bold tracking-tight mb-4">
                    Workspace <span className="text-gradient-primary">Manifest</span>
                </h2>
                <p className="mt-4 text-lg text-muted max-w-2xl mx-auto leading-relaxed font-body">
                    polyws supports JSON and TOML config files using .poly/.polyws naming variants.
                </p>
                <p className="mt-4 font-mono text-xs text-muted max-w-3xl mx-auto">
                    Valid names: .polyws, .poly, .polyws.json, .poly.json, .polyws.toml, .poly.toml
                </p>
                <div className="mt-6">
                    <div className="flex flex-col sm:flex-row gap-3 items-center justify-center">
                        <a
                            href={generatorUrl}
                            className="inline-flex items-center justify-center px-6 py-3 text-sm font-body font-semibold text-white uppercase tracking-wider bg-gradient-primary rounded-full shadow-glow-primary hover:shadow-glow-primary-hover hover:-translate-y-0.5 transition-all"
                        >
                            Open Config Generator
                        </a>
                        <a
                            href={docsUrl}
                            className="inline-flex items-center justify-center px-6 py-3 text-sm font-body font-semibold text-foreground uppercase tracking-wider border border-white/20 rounded-full hover:bg-white/5 hover:border-white/40 transition-all"
                        >
                            Open Docs
                        </a>
                    </div>
                </div>
            </div>
            
            <div className="flex flex-col lg:flex-row gap-10 max-w-6xl mx-auto px-4 relative z-10">
                {/* Annotated JSON */}
                <div className="flex-[3]">
                    <div className="group relative bg-[#030304] border border-white/10 rounded-xl overflow-hidden shadow-2xl">
                        <div className="flex items-center justify-between px-4 py-3 bg-white/5 border-b border-white/10">
                            <div className="flex items-center gap-2">
                                <div className="w-3 h-3 rounded-full bg-white/20"></div>
                                <div className="w-3 h-3 rounded-full bg-white/20"></div>
                                <div className="w-3 h-3 rounded-full bg-white/20"></div>
                                <span className="ml-2 font-mono text-xs text-muted">.polyws.json or .polyws.toml</span>
                            </div>
                            <button className="font-mono text-[10px] text-muted hover:text-primary transition uppercase tracking-widest px-2 py-1 bg-white/5 rounded border border-white/10 opacity-0 group-hover:opacity-100">Copy</button>
                        </div>
                        <div className="p-6 overflow-x-auto text-sm md:text-base">
                            <pre className="leading-relaxed font-mono text-foreground">
                                <span className="text-muted/60">{'{'}</span><br/>
                                {'  '}<span className="text-tertiary">"name"</span>: <span className="text-white">"rove"</span>,<br/>
                                {'  '}<span className="text-muted/60">{"// global mirror interval (minutes)"}</span><br/>
                                {'  '}<span className="text-tertiary">"sync_interval_minutes"</span>: <span className="text-primary">30</span>,<br/>
                                <br/>
                                {'  '}<span className="text-tertiary">"projects"</span>: [<br/>
                                {'    {'}<br/>
                                {'      '}<span className="text-tertiary">"name"</span>:     <span className="text-white">"core"</span>,<br/>
                                {'      '}<span className="text-tertiary">"url"</span>:      <span className="text-white">"git@github.com:org/core.git"</span>,<br/>
                                {'      '}<span className="text-tertiary">"branch"</span>:   <span className="text-white">"main"</span>,<br/>
                                {'      '}<span className="text-muted/60">{"// push --mirror to this remote"}</span><br/>
                                {'      '}<span className="text-tertiary">"sync_url"</span>: <span className="text-white">"git@gitlab.com:backup/core.git"</span><br/>
                                {'    },'}<br/>
                                {'    {'}<br/>
                                {'      '}<span className="text-tertiary">"name"</span>:       <span className="text-white">"plugins"</span>,<br/>
                                {'      '}<span className="text-tertiary">"url"</span>:        <span className="text-white">"git@github.com:org/plugins.git"</span>,<br/>
                                {'      '}<span className="text-tertiary">"branch"</span>:     <span className="text-white">"main"</span>,<br/>
                                {'      '}<span className="text-muted/60">{"// runs after \"core\" in exec + pull + push"}</span><br/>
                                {'      '}<span className="text-tertiary">"depends_on"</span>: [<span className="text-white">"core"</span>],<br/>
                                {'      '}<span className="text-tertiary">"sync_interval"</span>: <span className="text-primary">10</span><br/>
                                {'    }'}<br/>
                                {'  '}<span className="text-foreground">],</span><br/>
                                <br/>
                                {'  '}<span className="text-muted/60">{"// optional VM integration"}</span><br/>
                                {'  '}<span className="text-tertiary">"vm"</span>: {'{'}<br/>
                                {'    '}<span className="text-tertiary">"host"</span>: <span className="text-white">"azure-dev"</span>,<br/>
                                {'    '}<span className="text-tertiary">"user"</span>: <span className="text-white">"azureuser"</span>,<br/>
                                {'    '}<span className="text-tertiary">"path"</span>: <span className="text-white">"~/workspace/rove"</span>,<br/>
                                {'    '}<span className="text-tertiary">"sync"</span>: <span className="text-white">"mutagen"</span><br/>
                                {'  '}<span className="text-foreground">{"}"}</span><br/>
                                <span className="text-muted/60">{'}'}</span>
                            </pre>
                        </div>
                    </div>
                </div>
                
                {/* Field Descriptions */}
                <div className="flex-[2]">
                    <form className="bg-surface border border-white/5 rounded-2xl p-6 h-full font-mono text-sm shadow-[inset_0_0_50px_rgba(0,0,0,0.5)]">
                        <div className="font-heading font-semibold text-lg text-white mb-6 border-b border-white/10 pb-4">Schema Definition</div>
                        <ul className="space-y-4 text-muted">
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">name</span> 
                                <span className="text-xs text-white/50">Workspace identifier shown in TUI and status output</span>
                            </li>
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">sync_interval_minutes</span> 
                                <span className="text-xs text-white/50">Global default for the mirror daemon</span>
                            </li>
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">projects[].name</span> 
                                <span className="text-xs text-white/50">Local directory name (relative to workspace root)</span>
                            </li>
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">projects[].url</span> 
                                <span className="text-xs text-white/50">Git clone URL (SSH or HTTPS)</span>
                            </li>
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">projects[].depends_on</span> 
                                <span className="text-xs text-white/50">List of project names this repo depends on; drives execution ordering</span>
                            </li>
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">projects[].sync_url</span> 
                                <span className="text-xs text-white/50">Remote URL to mirror push to via sync daemon</span>
                            </li>
                            <li className="flex flex-col gap-1">
                                <span className="text-tertiary">vm.sync</span> 
                                <span className="text-xs text-white/50">Sync method selector (mutagen or rsync)</span>
                            </li>
                        </ul>
                    </form>
                </div>
            </div>
        </section>
    );
}
