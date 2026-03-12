import React from "react";
import logoUrl from "./assets/polyws_logo.min.png";

export default function Navbar() {
    const generatorUrl = `${import.meta.env.BASE_URL}generet/`;

    return (
        <nav className="fixed top-0 left-0 right-0 z-50 flex flex-col sm:flex-row items-center justify-between py-4 px-6 border-b border-white/10 bg-background/80 backdrop-blur-md gap-4 sm:gap-0">
            <div className="flex items-center gap-3">
                <img src={logoUrl} alt="polyws logo" className="h-6 sm:h-8 object-contain" />
                <span className="ml-2 text-xs font-mono text-muted tracking-widest uppercase hidden sm:inline-block">[v1.0.1]</span>
            </div>

            {/* Desktop Navigation */}
            <div className="hidden md:flex gap-8 font-mono text-sm uppercase tracking-wider items-center">
                <a href="#features" className="text-muted hover:text-white transition-colors">features</a>
                <a href="#commands" className="text-muted hover:text-white transition-colors">commands</a>
                <a href="#tui" className="text-muted hover:text-white transition-colors">tui</a>
                <a href="#config" className="text-muted hover:text-white transition-colors">config</a>
                <a href={generatorUrl} className="text-muted hover:text-white transition-colors">generet</a>
            </div>

            {/* Actions */}
            <div className="flex gap-4 items-center">
                <a href="https://github.com/cmdworks/polyws" target="_blank" rel="noopener" className="hidden sm:inline-flex items-center justify-center px-6 py-2 text-sm font-body font-medium text-foreground border border-white/20 rounded-full hover:bg-white/5 hover:border-white/40 transition-all">
                    GitHub
                </a>
                <a href={generatorUrl} className="hidden sm:inline-flex items-center justify-center px-6 py-2 text-sm font-body font-medium text-foreground border border-primary/40 rounded-full hover:bg-primary/10 hover:border-primary/70 transition-all">
                    Generator
                </a>
                <a href="#install" className="inline-flex items-center justify-center px-6 py-2 text-sm font-body font-semibold text-white uppercase tracking-wider bg-gradient-primary rounded-full shadow-glow-primary hover:shadow-glow-primary-hover hover:-translate-y-0.5 transition-all">
                    Get Started
                </a>
            </div>
        </nav>
    );
}
