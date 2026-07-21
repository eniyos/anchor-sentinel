"use client";

import { useEffect, useState, useCallback } from "react";

const WORDS = ["audit.", "detect.", "protect.", "scan.", "secure.", "verify.", "ship."];

type Platform = "cargo" | "linux" | "macos" | "windows";

const INSTALL: Record<Platform, { cmd: string; label: string }> = {
  cargo: {
    cmd: "cargo install anchor-sentinel",
    label: "Cargo (any)",
  },
  linux: {
    cmd: "curl -sL https://github.com/eniyos/anchor-sentinel/releases/latest/download/sentinel-x86_64-unknown-linux-gnu.tar.gz | tar xz && sudo mv sentinel /usr/local/bin/",
    label: "Linux (x86_64)",
  },
  macos: {
    cmd: "curl -sL https://github.com/eniyos/anchor-sentinel/releases/latest/download/sentinel-x86_64-apple-darwin.tar.gz | tar xz && sudo mv sentinel /usr/local/bin/",
    label: "macOS (x86_64)",
  },
  windows: {
    cmd: "curl -sLO https://github.com/eniyos/anchor-sentinel/releases/latest/download/sentinel-x86_64-pc-windows-msvc.zip && tar -xf sentinel-x86_64-pc-windows-msvc.zip",
    label: "Windows (x86_64)",
  },
};

function CopyIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

export default function HomePage() {
  const [platform, setPlatform] = useState<Platform>("cargo");
  const [copied, setCopied] = useState(false);
  const [flashing, setFlashing] = useState(false);

  useEffect(() => {
    document.documentElement.style.setProperty("--hue", "120");
    document.documentElement.style.setProperty("--start", "50vh");
    document.documentElement.style.setProperty("--space", "50vh");
  }, []);

  const current = INSTALL[platform];

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(current.cmd);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = current.cmd;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }

    setCopied(true);
    setFlashing(true);
    setTimeout(() => {
      setCopied(false);
      setTimeout(() => setFlashing(false), 300);
    }, 2000);
  }, [current.cmd]);

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
              onClick={() => { setPlatform(key); setCopied(false); }}
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

        {/* Terminal block — click to copy */}
        <button
          onClick={handleCopy}
          className={`terminal-block w-full max-w-2xl cursor-pointer group relative text-left ${
            flashing ? "terminal-flash" : ""
          }`}
          aria-label="Click to copy install command"
        >
          <span className="prompt select-none">$</span>
          <span className="truncate">{current.cmd}</span>

          {/* Copy / Check icon */}
          <span className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300">
            <span className={`icon-transition ${copied ? "icon-visible icon-check" : "icon-hidden icon-copy"}`}>
              <CheckIcon />
            </span>
            <span className={`icon-transition ${copied ? "icon-hidden" : "icon-visible"}`}>
              <CopyIcon />
            </span>
          </span>
        </button>

        {/* "Copied!" toast */}
        <div className={`toast ${copied ? "toast-visible" : ""}`}>
          <CheckIcon />
          Copied to clipboard
        </div>

        {/* Links */}
        <div className="flex items-center gap-4">
          <a
            href="https://crates.io/crates/anchor-sentinel"
            target="_blank"
            rel="noopener noreferrer"
            className="text-zinc-500 hover:text-[#a1f4a1] transition-colors text-sm font-mono"
          >
            crates.io
          </a>
          <span className="text-zinc-800">/</span>
          <a
            href="https://github.com/eniyos/anchor-sentinel"
            target="_blank"
            rel="noopener noreferrer"
            className="text-zinc-500 hover:text-[#a1f4a1] transition-colors text-sm font-mono"
          >
            GitHub
          </a>
          <span className="text-zinc-800">/</span>
          <a
            href="https://github.com/eniyos/anchor-sentinel#quickstart"
            target="_blank"
            rel="noopener noreferrer"
            className="text-zinc-500 hover:text-[#a1f4a1] transition-colors text-sm font-mono"
          >
            Quickstart →
          </a>
        </div>
      </div>
    </div>
  );
}
