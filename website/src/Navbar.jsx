import React, { useState } from "react";
import { useLocation } from "react-router-dom";
import logoUrl from "./assets/polyws_logo.min.png";

export default function Navbar() {
    const [menuOpen, setMenuOpen] = useState(false);
    const location = useLocation();
    const base = import.meta.env.BASE_URL;
    const generatorUrl = `${base}generet`;
    const docsUrl = `${base}docs`;
    const changelogUrl = `${base}changelog`;
    const isHome = location.pathname === "/";
    const isDocs = location.pathname.startsWith("/docs");
    const isChangelog = location.pathname.startsWith("/changelog");
    const isGenerator = location.pathname.startsWith("/generet");
    const sectionHref = (id) => (isHome ? `#${id}` : `${base}#${id}`);
    const navCls = (active) =>
        active
            ? "text-white border-b border-primary transition-colors"
            : "text-muted hover:text-white transition-colors";

    const menuItems = [
        { label: "features", href: sectionHref("features"), active: false },
        { label: "commands", href: sectionHref("commands"), active: false },
        { label: "tui", href: sectionHref("tui"), active: false },
        { label: "config", href: sectionHref("config"), active: false },
        { label: "generet", href: generatorUrl, active: isGenerator },
        { label: "docs", href: docsUrl, active: isDocs },
        { label: "changelog", href: changelogUrl, active: isChangelog },
    ];

    return (
        <nav className="fixed top-0 left-0 right-0 z-50 border-b border-white/10 bg-background/80 backdrop-blur-md px-4 sm:px-6 py-3">
            <div className="max-w-7xl mx-auto">
                <div className="flex items-center justify-between gap-3">
                    <a href={base} className="flex items-center gap-3">
                        <img src={logoUrl} alt="polyws logo" className="h-6 sm:h-8 object-contain" />
                        <span className="ml-1 text-xs font-mono text-muted tracking-widest uppercase hidden sm:inline-block">[v1.0.3]</span>
                    </a>

                    <div className="flex items-center gap-2 sm:gap-3">
                        <button
                            type="button"
                            className="md:hidden inline-flex items-center justify-center w-10 h-10 border border-white/20 rounded-lg text-foreground hover:bg-white/5"
                            aria-label="Toggle menu"
                            aria-expanded={menuOpen}
                            onClick={() => setMenuOpen((v) => !v)}
                        >
                            <span className="font-mono text-lg leading-none">{menuOpen ? "×" : "☰"}</span>
                        </button>
                    </div>
                </div>

                <div className="hidden md:flex items-center justify-end mt-3">
                    <div className="flex gap-8 font-mono text-sm uppercase tracking-wider items-center">
                        {menuItems.map((item) => (
                            <a key={item.label} href={item.href} className={navCls(item.active)}>
                                {item.label}
                            </a>
                        ))}
                    </div>
                </div>

                {menuOpen && (
                    <div className="md:hidden mt-3 border border-white/15 rounded-xl bg-black/35 p-3">
                        <div className="grid grid-cols-2 gap-2 font-mono text-xs uppercase tracking-wider">
                            {menuItems.map((item) => (
                                <a
                                    key={item.label}
                                    href={item.href}
                                    className={`px-2 py-2 rounded border border-white/10 ${item.active ? "text-white bg-white/10" : "text-muted hover:text-white"}`}
                                    onClick={() => setMenuOpen(false)}
                                >
                                    {item.label}
                                </a>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </nav>
    );
}
