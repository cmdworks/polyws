import React from "react";
import { useLocation } from "react-router-dom";
import logoUrl from "./assets/polyws_logo.min.png";

export default function Footer() {
    const location = useLocation();
    const base = import.meta.env.BASE_URL;
    const generatorUrl = `${base}generet`;
    const docsUrl = `${base}docs`;
    const isHome = location.pathname === "/";
    const sectionHref = (id) => (isHome ? `#${id}` : `${base}#${id}`);

    return (
        <footer className="footer mt-20 py-12 border-t border-white/10 bg-[#030304] relative overflow-hidden">
            {/* Ambient Lighting */}
            <div className="absolute top-0 right-1/4 w-[300px] h-[300px] bg-primary opacity-5 blur-[100px] rounded-full pointer-events-none -z-10"></div>
            
            <div className="flex flex-col md:flex-row md:justify-between md:items-center gap-8 max-w-7xl mx-auto px-4 relative z-10">
                <div className="flex flex-col gap-4">
                    <img src={logoUrl} alt="polyws logo" className="h-8 object-contain w-fit" />
                    <p className="text-muted text-sm font-body max-w-sm leading-relaxed">
                        Polyrepo workspace orchestrator. Built in Rust. Zero configuration to start. Production-grade from day one.
                    </p>
                </div>
                
                <nav className="flex flex-wrap gap-x-8 gap-y-4 text-xs font-mono uppercase tracking-widest text-muted" aria-label="Footer navigation">
                    <a href={sectionHref("features")} className="hover:text-primary transition-colors">Features</a>
                    <a href={sectionHref("commands")} className="hover:text-primary transition-colors">Commands</a>
                    <a href={sectionHref("tui")} className="hover:text-primary transition-colors">TUI</a>
                    <a href={sectionHref("config")} className="hover:text-primary transition-colors">Config</a>
                    <a href={generatorUrl} className="hover:text-primary transition-colors">Generet</a>
                    <a href={docsUrl} className="hover:text-primary transition-colors">Docs</a>
                    <a href={sectionHref("install")} className="hover:text-primary transition-colors">Install</a>
                    <a href="https://github.com/cmdworks/polyws" target="_blank" rel="noopener" className="text-white hover:text-tertiary transition-colors flex items-center gap-1">
                        GitHub
                        <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                    </a>
                </nav>
            </div>
            
            <div className="flex flex-col md:flex-row md:justify-between md:items-center gap-4 mt-12 pt-8 border-t border-white/5 max-w-7xl mx-auto px-4 text-xs font-mono text-muted/60 relative z-10">
                <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2 bg-white/5 px-2 py-1 rounded border border-white/5">
                        <div className="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></div>
                        <span className="text-primary tracking-widest uppercase">System Online</span>
                    </div>
                    <span>polyws © 2026</span>
                </div>
                <div className="flex items-center gap-4">
                    <span>MIT License</span>
                    <span className="hidden md:inline text-white/20">|</span>
                    <span className="flex items-center gap-2">
                        Engine: <span className="text-white">Rust 1.75</span>
                    </span>
                </div>
            </div>
        </footer>
    );
}
