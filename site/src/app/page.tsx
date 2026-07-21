"use client";

import { useEffect, useState } from "react";

const WORDS = ["audit.", "detect.", "protect.", "scan.", "secure.", "verify.", "ship."];

type Platform = "cargo" | "linux" | "macos" | "windows";

const INSTALL: Record<Platform, { cmd: string; label: string }> = {
  cargo: { cmd: "cargo install anchor-sentinel", label: "Cargo" },
  linux: { cmd: "curl -sL https://github.com/eniyos/anchor-sentinel/releases/latest/download/sentinel-x86_64-unknown-linux-gnu.tar.gz | tar xz && sudo mv sentinel /usr/local/bin/", label: "Linux" },
  macos: { cmd: "curl -sL https://github.com/eniyos/anchor-sentinel/releases/latest/download/sentinel-x86_64-apple-darwin.tar.gz | tar xz && sudo mv sentinel /usr/local/bin/", label: "macOS" },
  windows: { cmd: "curl -sLO https://github.com/eniyos/anchor-sentinel/releases/latest/download/sentinel-x86_64-pc-windows-msvc.zip && tar -xf sentinel-x86_64-pc-windows-msvc.zip", label: "Windows" },
};

export default function HomePage() {
  const [platform, setPlatform] = useState<Platform>("cargo");

  useEffect(() => {
    document.documentElement.style.setProperty("--hue", "120");
    document.documentElement.style.setProperty("--start", "50vh");
    document.documentElement.style.setProperty("--space", "50vh");
  }, []);

  const current = INSTALL[platform];

  return (
    <div
      className="min-h-screen w-screen"
      style={{ ["--count" as string]: WORDS.length } as React.CSSProperties}
    >
      <header className="sentinel-header">
        <section>
          <h1 className="sr-only">Anchor Sentinel — static security analysis for Solana programs.</h1>
          <ul className="sentinel-words" aria-hidden="true">
            {WORDS.map((word, i) => (
              <li key={i} style={{ ["--i" as string]: i } as React.CSSProperties}>
                {word}
              </li>
            ))}
          </ul>
        </section>
      </header>

      <div className="flex flex-col items-center gap-6 w-full px-4 pt-8 pb-24">
        {/* Platform tabs */}
        <div className="flex gap-1 bg-zinc-900 rounded-xl p-1 border border-zinc-800">
          {(Object.entries(INSTALL) as [Platform, typeof current][]).map(([key, val]) => (
            <button
              key={key}
              onClick={() => setPlatform(key)}
              className={`px-4 py-2 rounded-lg text-sm font-mono transition-all ${
                platform === key
                  ? "bg-[#a1f4a1]/10 text-[#a1f4a1] border border-[#a1f4a1]/30"
                  : "text-zinc-500 hover:text-zinc-300 border border-transparent"
              }`}
            >
              {val.label}
            </button>
          ))}
        </div>

        {/* Terminal command block */}
        <div className="terminal-block w-full max-w-2xl text-left select-all">
          <span className="prompt select-none">$</span>
          <span className="truncate">{current.cmd}</span>
          <span className="copy-hint">copy</span>
        </div>

        {/* Links */}
        <div className="flex items-center gap-4">
          <a href="https://crates.io/crates/anchor-sentinel" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-[#a1f4a1] transition-colors text-sm font-mono">crates.io</a>
          <span className="text-zinc-800">/</span>
          <a href="https://github.com/eniyos/anchor-sentinel" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-[#a1f4a1] transition-colors text-sm font-mono">GitHub</a>
          <span className="text-zinc-800">/</span>
          <a href="https://github.com/eniyos/anchor-sentinel#quickstart" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-[#a1f4a1] transition-colors text-sm font-mono">Quickstart →</a>
        </div>
      </div>
    </div>
  );
}
