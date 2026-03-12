import React from "react";
import iconUrl from "./assets/polyws_logo.icon.png";
import Navbar from "./Navbar";
import FeaturesSection from "./FeaturesSection";
import CommandsSection from "./CommandsSection";
import HowSection from "./HowSection";
import TuiSection from "./TuiSection";
import ConfigSection from "./ConfigSection";
import InstallSection from "./InstallSection";
import Footer from "./Footer";

export default function App() {
  return (
    <div className="min-h-screen bg-background text-foreground font-body relative overflow-x-hidden">
      <Navbar />

      <main className="max-w-7xl mx-auto px-4 py-8">
        <section id="hero" className="relative py-24 md:py-32 flex flex-col items-center text-center min-h-[80vh] overflow-hidden">

          {/* Ambient Radial Glow */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-primary opacity-10 blur-[150px] rounded-full pointer-events-none"></div>

          {/* Grid background overlay */}
          <div className="absolute inset-x-0 bottom-0 h-full bg-grid-pattern opacity-50 z-0 pointer-events-none mask-image-bottom"></div>

          <div className="relative z-10 flex flex-col gap-8 items-center w-full max-w-4xl mx-auto">

            {/* Live Status Badge */}
            <div className="flex items-center gap-3 mb-4 px-4 py-2 rounded-full border border-white/10 bg-surface/50 backdrop-blur-md">
              <span className="relative flex h-2.5 w-2.5">
                <span className="animate-ping-slow absolute inline-flex h-full w-full rounded-full bg-gradient-primary opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-gradient-primary"></span>
              </span>
              <span className="font-mono text-muted text-xs uppercase tracking-wider">Built in Rust · Zero Dependencies</span>
            </div>

            {/* Headline */}
            <h1 className="font-heading font-bold tracking-tight text-5xl sm:text-6xl md:text-8xl leading-[1.1] mb-2">
              <span className="block text-foreground">Polyrepo Workspace </span>
              <span className="block text-gradient-primary">Orchestrator</span>
            </h1>

            {/* Subheadline */}
            <p className="font-body text-muted text-lg md:text-xl max-w-2xl leading-relaxed mb-6">
              polyws is a production-grade CLI tool for managing multiple Git repositories as a unified workspace. Dependency-aware parallel execution, snapshot & restore, mirror daemons, VM integration — and a full interactive TUI, all from a single binary.
            </p>

            {/* CTA buttons */}
            <div className="flex flex-col sm:flex-row gap-4 w-full justify-center">
              <a href="#install" className="group relative inline-flex items-center justify-center px-8 py-4 text-base font-body font-semibold text-white uppercase tracking-wider bg-gradient-primary rounded-full shadow-glow-primary hover:shadow-glow-primary-hover hover:-translate-y-0.5 transition-all duration-300 min-w-[200px]">
                Install polyws
                <svg className="ml-2 w-4 h-4 group-hover:translate-x-1 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14 5l7 7m0 0l-7 7m7-7H3" />
                </svg>
              </a>
              <a href="https://github.com/cmdworks/polyws" target="_blank" rel="noopener noreferrer" className="inline-flex items-center justify-center px-8 py-4 text-base font-body font-medium text-foreground border border-white/20 rounded-full hover:bg-white/5 hover:border-white/40 transition-all duration-300 min-w-[200px]">
                View on GitHub
              </a>
            </div>

            {/* 3D Orb / Abstract Floating Graphic */}
            <div className="mt-16 relative w-full h-[300px] md:h-[450px] animate-float flex justify-center items-center">
              {/* Outer Ring — gradient circle border */}
              <div
                className="absolute w-[280px] h-[280px] md:w-[400px] md:h-[400px] rounded-full animate-spin-slow p-[2px]"
                style={{ background: "linear-gradient(135deg, #00D2FF, #3A7BD5, #805AD7, #FF42A1)" }}
              >
                <div className="w-full h-full rounded-full" style={{ background: "#030304" }} />
              </div>
              {/* Inner Ring */}
              <div className="absolute w-[200px] h-[200px] md:w-[280px] md:h-[280px] border-2 border-dashed border-tertiary/30 rounded-full animate-spin-reverse"></div>
              {/* Core */}
              <div className="absolute w-[120px] h-[120px] md:w-[160px] md:h-[160px] bg-gradient-primary rounded-full blur-[2px] shadow-glow-primary"></div>
              <div className="absolute w-[100px] h-[100px] md:w-[140px] md:h-[140px] bg-surface rounded-full border border-white/10 flex items-center justify-center z-10 glass-panel">
                <img src={iconUrl} alt="polyws icon" className="w-12 h-12 md:w-16 md:h-16 object-contain" />
              </div>

              {/* Floating Stat Cards */}
              <div className="absolute top-10 right-[10%] md:right-[20%] glass-panel rounded-xl p-4 animate-bounce" style={{ animationDuration: '4s' }}>
                <p className="font-mono text-xs text-muted mb-1 uppercase">Parallel_Threads</p>
                <p className="font-heading font-bold text-lg text-gradient-primary">16</p>
              </div>
              <div className="absolute bottom-20 left-[5%] md:left-[15%] glass-panel rounded-xl p-4 animate-bounce" style={{ animationDuration: '3.5s', animationDelay: '1s' }}>
                <p className="font-mono text-xs text-muted mb-1 uppercase">Repos_Tracked</p>
                <p className="font-heading font-bold text-lg text-gradient-primary">12</p>
              </div>
            </div>

          </div>
        </section>

        <FeaturesSection />
        <TuiSection />
        <HowSection />
        <CommandsSection />
        <ConfigSection />
        <InstallSection />
        <Footer />
      </main>
    </div>
  );
}
