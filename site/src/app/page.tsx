"use client";

import { useEffect } from "react";

const WORDS = ["audit.", "detect.", "protect.", "scan.", "secure.", "verify.", "ship."];
const FEATURES = [
  { severity: "critical", color: "#ef4444", label: "Critical", rules: ["cpi_signer_seed_validation", "missing_balance_check", "missing_signer"] },
  { severity: "high", color: "#eab308", label: "High", rules: ["duplicate_mutable_accounts", "lamports_drain", "missing_bump_seed_canonicalization", "missing_close_authority", "missing_ownership", "missing_reinit_guard", "pda_misconfig"] },
  { severity: "medium", color: "#3b82f6", label: "Medium", rules: ["integer_cast_truncation", "missing_mut", "unchecked_balance_flow", "unsafe_arithmetic"] },
];
const ALL_RULES = [...FEATURES.flatMap((f) => f.rules)];

export default function HomePage() {
  useEffect(() => {
    document.documentElement.style.setProperty("--hue", "270");
    document.documentElement.style.setProperty("--start", "50vh");
    document.documentElement.style.setProperty("--space", "50vh");
  }, []);

  return (
    <>
      {/* Hero — sticky word cycling */}
      <div
        className="min-h-screen w-screen"
        style={{ ["--count" as string]: WORDS.length } as React.CSSProperties}
      >
        <header className="sentinel-header">
          <section>
            <h1 className="sr-only sm:not-sr-only">
              <span aria-hidden="true">you can&nbsp;</span>
              <span className="sr-only">Anchor Sentinel — static security analysis for Solana programs.</span>
            </h1>
            <ul className="sentinel-words" aria-hidden="true">
              {WORDS.map((word, i) => (
                <li key={i} style={{ ["--i" as string]: i } as React.CSSProperties}>
                  {word}
                </li>
              ))}
            </ul>
          </section>
        </header>

        <main className="sentinel-main anim-section">
          <div className="max-w-6xl mx-auto space-y-24">
            {/* Hero install block */}
            <section className="text-center pt-12">
              <div className="flex justify-center mb-6">
                <div className="terminal-block">
                  <span className="prompt">$</span>
                  <span>cargo install anchor-sentinel</span>
                </div>
              </div>
              <p className="text-xl text-zinc-400 max-w-2xl mx-auto">
                Static security analysis for Solana Anchor programs.
                <br />
                14 rules, 8 of 9 Sealevel-Attacks classes covered.
              </p>
              <div className="flex justify-center gap-4 mt-8 text-sm">
                <a
                  href="https://crates.io/crates/anchor-sentinel"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="px-5 py-2 rounded-full border border-zinc-700 text-zinc-300 hover:bg-zinc-900 transition-colors"
                >
                  crates.io
                </a>
                <a
                  href="https://github.com/eniyos/anchor-sentinel"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="px-5 py-2 rounded-full border border-zinc-700 text-zinc-300 hover:bg-zinc-900 transition-colors"
                >
                  GitHub
                </a>
                <a
                  href="https://github.com/eniyos/anchor-sentinel#quickstart"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="px-5 py-2 rounded-full border border-zinc-700 text-zinc-300 hover:bg-zinc-900 transition-colors"
                >
                  Quickstart →
                </a>
              </div>
            </section>

            {/* How it works */}
            <section>
              <h2 className="text-2xl font-semibold mb-8 text-center">
                <span className="text-zinc-500">$</span> sentinel scan .
              </h2>
              <div className="feature-grid">
                <div className="anim-card feature-card">
                  <h3 className="font-medium text-zinc-200 mb-2">IDL + AST Analysis</h3>
                  <p className="text-sm text-zinc-500 leading-relaxed">
                    Two-layer scanning. IDL checks account metadata. AST parsing detects unsafe
                    arithmetic, missing balance checks, and CPI seed vulnerabilities in source.
                  </p>
                </div>
                <div className="anim-card feature-card">
                  <h3 className="font-medium text-zinc-200 mb-2">CI-Ready</h3>
                  <p className="text-sm text-zinc-500 leading-relaxed">
                    SARIF output for GitHub Code Scanning. Pre-built binaries. Fail builds on
                    critical findings. Works in any CI pipeline.
                  </p>
                </div>
                <div className="anim-card feature-card">
                  <h3 className="font-medium text-zinc-200 mb-2">Educational</h3>
                  <p className="text-sm text-zinc-500 leading-relaxed">
                    <code className="text-[#a1f4a1]">sentinel explain &lt;rule&gt;</code> teaches why
                    each pattern is dangerous with vulnerable and safe code examples.
                  </p>
                </div>
              </div>
            </section>

            {/* Rules catalog */}
            <section>
              <h2 className="text-2xl font-semibold mb-2 text-center">14 Security Rules</h2>
              <p className="text-zinc-500 text-center mb-10 text-sm">
                Covering 8 of 9 canonical Sealevel-Attacks classes
              </p>

              {FEATURES.map((group) => (
                <div key={group.severity} className="mb-10">
                  <h3 className="text-sm font-medium uppercase tracking-widest mb-4 flex items-center gap-2">
                    <span
                      className="inline-block w-2 h-2 rounded-full"
                      style={{ background: group.color }}
                    />
                    {group.label}
                    <span className="text-zinc-600 font-normal">— {group.rules.length} rules</span>
                  </h3>
                  <div className="feature-grid">
                    {group.rules.map((rule) => (
                      <div key={rule} className="anim-card feature-card">
                        <code className="text-sm font-mono" style={{ color: group.color }}>
                          {rule}
                        </code>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </section>

            {/* CTA */}
            <section className="text-center pb-12">
              <h2 className="text-xl font-semibold mb-4">Deploy with confidence</h2>
              <div className="flex justify-center">
                <div className="terminal-block">
                  <span className="prompt">$</span>
                  <span>sentinel scan .</span>
                </div>
              </div>
              <p className="text-zinc-500 text-sm mt-6">
                <a
                  href="https://github.com/eniyos/anchor-sentinel"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline underline-offset-2 hover:text-zinc-300"
                >
                  GitHub
                </a>
                <span className="mx-3">·</span>
                <a
                  href="https://crates.io/crates/anchor-sentinel"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline underline-offset-2 hover:text-zinc-300"
                >
                  crates.io
                </a>
                <span className="mx-3">·</span>
                <a
                  href="https://github.com/eniyos/anchor-sentinel#rules"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline underline-offset-2 hover:text-zinc-300"
                >
                  Docs
                </a>
              </p>
            </section>
          </div>

          {/* Footer */}
          <footer className="text-center text-xs text-zinc-700 pt-8 border-t border-zinc-900 mt-12 max-w-6xl mx-auto">
            Anchor Sentinel — MIT / Apache-2.0
          </footer>
        </main>
      </div>
    </>
  );
}
