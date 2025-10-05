---
layout: default
title: "Mica Language"
nav_order: 1
---

<style>
:root {
  --mica-blue: #1a5fb4;
  --mica-ink: #0b1526;
  --mica-slate: #4a5b76;
  --mica-cloud: #f5f8ff;
  --mica-white: #ffffff;
  --mica-lilac: #b097ff;
  --mica-teal: #22b8cf;
  --mica-amber: #ffc857;
}

.page-wrapper {
  font-family: "Inter", "Segoe UI", -apple-system, BlinkMacSystemFont, "Helvetica Neue", sans-serif;
  color: var(--mica-ink);
}

.page-wrapper a {
  color: var(--mica-blue);
  font-weight: 600;
}

.hero {
  position: relative;
  margin: -2rem -2rem 3rem;
  padding: 4rem 2rem 3.5rem;
  background: radial-gradient(circle at 15% 20%, rgba(26, 95, 180, 0.16), transparent 50%),
    radial-gradient(circle at 85% 0%, rgba(15, 64, 130, 0.2), transparent 60%),
    linear-gradient(135deg, #0f2647 0%, #122f61 35%, #1a5fb4 100%);
  color: var(--mica-white);
  border-radius: 24px;
  overflow: hidden;
}

.hero::before {
  content: "";
  position: absolute;
  inset: -20%;
  background: conic-gradient(from 120deg, rgba(34, 184, 207, 0.35), rgba(176, 151, 255, 0.15), transparent 60%);
  filter: blur(60px);
  animation: heroGlow 18s infinite linear;
}

.hero::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), transparent 55%);
  pointer-events: none;
}

.hero-orbits {
  position: absolute;
  inset: 0;
  pointer-events: none;
  opacity: 0.45;
}

.hero-orbit {
  position: absolute;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.18);
  animation: orbit 24s linear infinite;
}

.hero-orbit:nth-child(1) {
  width: 55%;
  height: 55%;
  top: 12%;
  right: -10%;
}

.hero-orbit:nth-child(2) {
  width: 35%;
  height: 35%;
  bottom: -12%;
  left: 10%;
  animation-duration: 32s;
}

.hero-orbit:nth-child(3) {
  width: 18%;
  height: 18%;
  top: 18%;
  left: 22%;
  animation-duration: 16s;
}

.hero-content {
  max-width: 680px;
  position: relative;
  z-index: 2;
}

.hero-badges {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 1.2rem;
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.45rem 0.85rem;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
  border: 1px solid rgba(255, 255, 255, 0.24);
  font-size: 0.78rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.hero-eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.2em;
  font-size: 0.8rem;
  font-weight: 700;
  opacity: 0.8;
}

.hero h1 {
  margin: 0.6rem 0 1rem;
  font-size: clamp(2.4rem, 4vw, 3.2rem);
  line-height: 1.15;
}

.hero p {
  font-size: 1.15rem;
  line-height: 1.6;
  opacity: 0.92;
}

.hero-marquee {
  margin-top: 1.5rem;
  overflow: hidden;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(5, 16, 38, 0.55);
}

.hero-marquee-track {
  display: flex;
  gap: 2.5rem;
  padding: 0.75rem 1.25rem;
  animation: marquee 22s linear infinite;
  font-size: 0.92rem;
  white-space: nowrap;
}

.hero-demo {
  position: relative;
  margin-top: 2.5rem;
  padding: 1.25rem 1.5rem;
  background: rgba(8, 19, 40, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 16px;
  box-shadow: 0 35px 65px rgba(5, 16, 38, 0.35);
  font-family: "Fira Code", "SFMono-Regular", Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 0.95rem;
}

.hero-demo[data-animated="true"] pre {
  margin: 0;
  color: var(--mica-white);
}

.hero-demo[data-animated="true"] .typing-line {
  opacity: 0.85;
  display: block;
  min-height: 1.1em;
}

.hero-demo[data-animated="true"] .typing-line.active {
  color: var(--mica-teal);
  opacity: 1;
}

.hero-demo .prompt {
  color: #7ac4ff;
}

.hero-demo .comment {
  color: #9abbe6;
}

.cta-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin: 2rem 0 0;
}

.cta-buttons .button {
  background: rgba(255, 255, 255, 0.12);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: 999px;
  color: var(--mica-white);
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-weight: 600;
  padding: 0.65rem 1.5rem;
  text-decoration: none;
  transition: transform 0.15s ease, background-color 0.2s ease, border-color 0.2s ease;
}

.cta-buttons .button:hover {
  transform: translateY(-1px);
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.5);
}

.section {
  margin: 0 0 4rem;
  padding: 0 0 0.5rem;
}

.section h2 {
  font-size: clamp(1.9rem, 3vw, 2.4rem);
  margin-bottom: 1rem;
}

.section p.lead {
  font-size: 1.1rem;
  color: var(--mica-slate);
  max-width: 60ch;
}

.section .section-eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.18em;
  font-size: 0.75rem;
  color: var(--mica-blue);
  font-weight: 700;
  margin-bottom: 0.6rem;
}

.metric-cards,
.command-grid,
.feature-grid,
.example-grid,
.resource-grid {
  display: grid;
  gap: 1.4rem;
}

.metric-cards {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-top: 2rem;
}

.metric-card {
  background: var(--mica-cloud);
  border-radius: 16px;
  padding: 1.25rem 1.4rem;
  border: 1px solid rgba(26, 95, 180, 0.12);
  box-shadow: 0 15px 30px rgba(15, 41, 80, 0.08);
}

.metric-card .label {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--mica-slate);
}

.metric-card .value {
  font-size: 1.7rem;
  font-weight: 700;
  color: var(--mica-ink);
  margin-top: 0.4rem;
}

.command-grid {
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  margin-top: 2rem;
}

.command-card {
  background: var(--mica-white);
  border-radius: 18px;
  padding: 1.4rem;
  border: 1px solid rgba(26, 95, 180, 0.15);
  box-shadow: 0 25px 45px rgba(12, 32, 64, 0.08);
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  position: relative;
  overflow: hidden;
}

.command-card::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(130deg, rgba(34, 184, 207, 0.08), transparent 55%);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.command-card:hover::after {
  opacity: 1;
}

.command-card h3 {
  margin: 0;
  font-size: 1.1rem;
}

.command-card pre {
  background: var(--mica-cloud);
  border-radius: 12px;
  padding: 0.9rem 1rem;
  margin: 0;
  overflow-x: auto;
  font-size: 0.9rem;
}

.feature-grid {
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  margin-top: 2.2rem;
}

.feature-card {
  background: var(--mica-white);
  border: 1px solid rgba(26, 95, 180, 0.15);
  border-radius: 20px;
  padding: 1.6rem;
  box-shadow: 0 20px 40px rgba(10, 30, 60, 0.1);
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.feature-card h3 {
  margin: 0;
}

.feature-card:hover {
  transform: translateY(-6px);
  box-shadow: 0 30px 55px rgba(10, 30, 60, 0.18);
}

.timeline {
  position: relative;
  margin-top: 2.5rem;
  padding-left: 1rem;
  border-left: 2px solid rgba(26, 95, 180, 0.2);
}

.timeline-step {
  position: relative;
  margin-bottom: 1.9rem;
  padding-left: 1.6rem;
}

.timeline-step::before {
  content: "";
  position: absolute;
  left: -1.15rem;
  top: 0.3rem;
  width: 0.75rem;
  height: 0.75rem;
  border-radius: 50%;
  background: var(--mica-blue);
  box-shadow: 0 0 0 6px rgba(26, 95, 180, 0.15);
}

.timeline-step:last-child {
  margin-bottom: 0;
}

.timeline-step code {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  background: var(--mica-cloud);
  border-radius: 8px;
  padding: 0.35rem 0.6rem;
  font-size: 0.85rem;
}

.example-grid {
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  margin-top: 2.2rem;
}

.example-card {
  background: linear-gradient(145deg, var(--mica-white), rgba(26, 95, 180, 0.08));
  border: 1px solid rgba(26, 95, 180, 0.12);
  border-radius: 18px;
  padding: 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
  position: relative;
  overflow: hidden;
}

.example-card h3 {
  margin: 0;
  font-size: 1.05rem;
}

.example-card footer {
  margin-top: auto;
  font-size: 0.9rem;
}

.example-card::after {
  content: "";
  position: absolute;
  inset: -60% 20% auto;
  height: 200%;
  background: radial-gradient(circle, rgba(255, 255, 255, 0.55), transparent 55%);
  opacity: 0;
  transition: opacity 0.3s ease, transform 0.35s ease;
}

.example-card:hover::after {
  opacity: 1;
  transform: translateY(-12%);
}

.resource-grid {
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  margin-top: 1.8rem;
}

.resource-card {
  background: var(--mica-cloud);
  border-radius: 16px;
  padding: 1.25rem 1.35rem;
  border: 1px solid rgba(26, 95, 180, 0.12);
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.resource-card strong {
  font-size: 1.05rem;
}

.highlight-block {
  background: linear-gradient(120deg, rgba(26, 95, 180, 0.12), rgba(26, 95, 180, 0.02));
  border-radius: 18px;
  padding: 1.75rem;
  border: 1px solid rgba(26, 95, 180, 0.15);
  display: grid;
  gap: 1rem;
  margin-top: 2.5rem;
}

.lab {
  margin-top: 3rem;
  background: var(--mica-white);
  border-radius: 20px;
  border: 1px solid rgba(26, 95, 180, 0.18);
  box-shadow: 0 35px 70px rgba(15, 30, 60, 0.12);
  overflow: hidden;
}

.lab-tabs {
  display: flex;
  flex-wrap: wrap;
  border-bottom: 1px solid rgba(26, 95, 180, 0.12);
  background: linear-gradient(120deg, rgba(176, 151, 255, 0.18), rgba(34, 184, 207, 0.15));
}

.lab-tabs label {
  flex: 1 1 140px;
  padding: 0.9rem 1.1rem;
  text-align: center;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s ease;
}

.lab-tabs label:hover {
  background: rgba(255, 255, 255, 0.35);
}

.lab input[type="radio"] {
  display: none;
}

.lab-panel {
  display: none;
  padding: 1.5rem 1.75rem 2rem;
  gap: 1.25rem;
  background: var(--mica-cloud);
}

.lab-panel h3 {
  margin: 0;
}

.lab-panel pre {
  background: var(--mica-white);
  border-radius: 12px;
  border: 1px solid rgba(26, 95, 180, 0.15);
  padding: 1rem 1.2rem;
  overflow-x: auto;
  margin: 0;
  font-size: 0.9rem;
}

.lab input[type="radio"]:checked + label {
  background: rgba(255, 255, 255, 0.5);
  box-shadow: inset 0 -3px 0 var(--mica-blue);
}

.lab input[type="radio"]:checked + label + .lab-panel {
  display: grid;
}

.mosaic {
  margin-top: 3rem;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1.4rem;
}

.mosaic-card {
  position: relative;
  border-radius: 18px;
  overflow: hidden;
  padding: 1.6rem 1.4rem 1.8rem;
  background: radial-gradient(circle at top left, rgba(26, 95, 180, 0.18), transparent 65%), var(--mica-ink);
  color: var(--mica-white);
  min-height: 200px;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  box-shadow: 0 35px 60px rgba(10, 20, 45, 0.25);
  transition: transform 0.25s ease;
}

.mosaic-card:hover {
  transform: translateY(-8px);
}

.mosaic-card::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(140deg, rgba(255, 200, 87, 0.2), transparent 60%);
  mix-blend-mode: screen;
}

.mosaic-card strong {
  font-size: 1.05rem;
  z-index: 1;
}

.mosaic-card p,
.mosaic-card footer {
  z-index: 1;
}

.mosaic-card footer {
  margin-top: auto;
  font-size: 0.85rem;
  opacity: 0.8;
}

.badge-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin-top: 1.6rem;
}

.badge-strip span {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.45rem 0.75rem;
  border-radius: 999px;
  background: rgba(26, 95, 180, 0.12);
  border: 1px solid rgba(26, 95, 180, 0.2);
  font-weight: 600;
  font-size: 0.8rem;
}

.badge-strip span::before {
  content: "✦";
  color: var(--mica-blue);
}

.sparkle-cta {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.7rem 1.4rem;
  border-radius: 999px;
  background: linear-gradient(120deg, rgba(26, 95, 180, 0.92), rgba(176, 151, 255, 0.85));
  color: var(--mica-white);
  font-weight: 700;
  text-decoration: none;
  box-shadow: 0 18px 35px rgba(26, 95, 180, 0.35);
}

.sparkle-cta::after {
  content: "✨";
  animation: twinkle 2.2s ease-in-out infinite;
}

.footer-note {
  margin: 4rem 0 1rem;
  text-align: center;
  color: var(--mica-slate);
  font-size: 0.95rem;
}

@keyframes heroGlow {
  0% {
    transform: rotate(0deg) scale(1.05);
  }
  50% {
    transform: rotate(180deg) scale(1.08);
  }
  100% {
    transform: rotate(360deg) scale(1.05);
  }
}

@keyframes orbit {
  from {
    transform: rotate(0deg) translateX(0);
  }
  to {
    transform: rotate(360deg) translateX(0);
  }
}

@keyframes marquee {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(-50%);
  }
}

@keyframes twinkle {
  0%,
  100% {
    opacity: 0.5;
    transform: translateY(0);
  }
  50% {
    opacity: 1;
    transform: translateY(-2px);
  }
}

@media (max-width: 720px) {
  .hero {
    margin: -1.5rem -1.5rem 3rem;
    padding: 3rem 1.5rem 3rem;
  }

  .hero-demo {
    font-size: 0.85rem;
  }

  .hero-marquee-track {
    animation-duration: 28s;
  }

  .lab-tabs label {
    flex: 1 1 100%;
  }
}
</style>

<div class="page-wrapper">
  <section class="hero" id="overview">
    <div class="hero-orbits">
      <div class="hero-orbit"></div>
      <div class="hero-orbit"></div>
      <div class="hero-orbit"></div>
    </div>
    <div class="hero-content">
      <div class="hero-badges">
        <span class="hero-badge">Single-binary pipeline</span>
        <span class="hero-badge">Snapshot verified</span>
        <span class="hero-badge">Weekend-sized</span>
      </div>
      <div class="hero-eyebrow">Minimal • Industrial • Composable • Auditable</div>
      <h1>Build industrial-grade language tooling without the million-line overhead.</h1>
      <p>
        Mica is a compact systems language prototype that exposes the full compiler pipeline—
        lexer to native backend—through one approachable repository. The README, GitHub Pages
        tour, and runnable examples all regenerate from real CLI output, so what you read is what
        the binary actually does.
      </p>
      <div class="hero-marquee">
        <div class="hero-marquee-track">
          <span>✨ Snapshot-backed docs stay honest.</span>
          <span>🧪 Deterministic providers keep traces stable.</span>
          <span>🛠️ Extend passes with familiar Rust ergonomics.</span>
          <span>🧱 Teach compilers without scaffolding a runtime.</span>
          <span>✨ Snapshot-backed docs stay honest.</span>
          <span>🧪 Deterministic providers keep traces stable.</span>
          <span>🛠️ Extend passes with familiar Rust ergonomics.</span>
          <span>🧱 Teach compilers without scaffolding a runtime.</span>
        </div>
      </div>
      <div class="cta-buttons">
        <a class="button" href="https://github.com/mica-lang/mica">View the repository ↗</a>
        <a class="button" href="https://mica-lang.github.io/mica/">Open the published docs site ↗</a>
        <a class="button" href="https://github.com/mica-lang/mica/blob/main/README.md#quickstart">Read the README quickstart ↗</a>
        <a class="button" href="https://github.com/search?q=repo%3Amica-lang%2Fmica+path%3Aexamples&type=code">Browse runnable examples</a>
      </div>
      <div class="hero-demo" data-animated="true">
        <pre><code><span class="typing-line" data-command="# Explore every compiler stage with one binary"></span>
<span class="typing-line" data-command="cargo run --bin mica -- --ast --pretty examples/adt.mica"></span>
<span class="typing-line" data-command="cargo run --bin mica -- --lower examples/methods.mica"></span>
<span class="typing-line" data-command="cargo run --bin mica -- --run --trace-json - examples/concurrency_pipeline.mica"></span></code></pre>
      </div>
    </div>
  </section>

  <section class="section" id="quickstart">
    <h2>Get hands-on in three steps</h2>
    <p class="lead">
      Clone, build, and explore in minutes. Mica ships with runnable samples, snapshot-tested
      documentation, and deterministic concurrency demos ready to execute from the CLI.
    </p>
    <div class="command-grid">
      <div class="command-card">
        <h3>1. Clone & build</h3>
        <p>Grab the repository and build the `mica` CLI.</p>
        <pre><code>git clone https://github.com/mica-lang/mica.git
cd mica
cargo build</code></pre>
      </div>
      <div class="command-card">
        <h3>2. Smoke-test the toolchain</h3>
        <p>Run the full suite once so snapshots and examples stay trustworthy.</p>
        <pre><code>cargo test
cargo run --bin gen_snippets -- --check</code></pre>
      </div>
      <div class="command-card">
        <h3>3. Inspect the pipeline</h3>
        <p>Use CLI flags to peek behind each compiler stage.</p>
        <pre><code>cargo run --bin mica -- --tokens examples/adt.mica
cargo run --bin mica -- --ir examples/methods.mica</code></pre>
      </div>
    </div>
  </section>

  <section class="section" id="audience">
    <div class="section-eyebrow">Who it’s for</div>
    <h2>Tailored for curious language builders</h2>
    <p class="lead">
      Whether you’re prototyping a research idea or guiding a study group, Mica keeps the entire compiler
      pipeline visible without overwhelming you with infrastructure.
    </p>
    <div class="feature-grid">
      <div class="feature-card">
        <h3>Language tinkerers</h3>
        <p>Inspect lexing, parsing, semantic analysis, lowering, and codegen in one place—no runtime scaffolding required.</p>
      </div>
      <div class="feature-card">
        <h3>Educators & study groups</h3>
        <p>Teach from audited examples and snapshot-backed docs that ensure every command stays up to date.</p>
      </div>
      <div class="feature-card">
        <h3>Systems programmers</h3>
        <p>Experiment with deterministic concurrency, capability tracking, and SSA-based passes in a familiar Rust codebase.</p>
      </div>
    </div>
  </section>

  <section class="section" id="pillars">
    <h2>What makes Mica different?</h2>
    <p class="lead">
      Designed for language tinkerers and teaching teams, Mica balances industrial ergonomics with a
      weekend-sized codebase. Audit effects, reason about concurrency, and ship documentation that
      never drifts from real output.
    </p>
    <div class="feature-grid">
      <div class="feature-card">
        <h3>Auditable capability system</h3>
        <p>
          Explicit `using` clauses and capability rows ensure IO, resource access, and concurrency
          effects are visible and enforced at compile time. Snapshot tests back every CLI example.
        </p>
        <a href="https://github.com/mica-lang/mica/blob/main/docs/tour.md#effects">See the effect tour ↗</a>
      </div>
      <div class="feature-card">
        <h3>Deterministic concurrency primitives</h3>
        <p>
          Structured `spawn`/`await` lets you orchestrate pipelines without sacrificing reproducibility.
          Deterministic providers keep traces stable for teaching and debugging.
        </p>
        <a href="https://github.com/mica-lang/mica/blob/main/examples/concurrency_pipeline.mica">Inspect the pipeline example ↗</a>
      </div>
      <div class="feature-card">
        <h3>Every stage, one CLI</h3>
        <p>
          Lex, parse, resolve, lower, inspect typed SSA, emit LLVM, and run native code without leaving
          the repo. A single binary exposes the whole pipeline with friendly diagnostics.
        </p>
        <a href="https://github.com/mica-lang/mica/blob/main/docs/snippets.md">Browse CLI snapshots ↗</a>
      </div>
      <div class="feature-card">
        <h3>Snapshot-backed documentation</h3>
        <p>
          Tutorials regenerate from real commands via `cargo run --bin gen_snippets`. Fresh output keeps
          lessons and screenshots evergreen.
        </p>
        <a href="https://github.com/mica-lang/mica/blob/main/docs/status_summary.md">Track verification coverage ↗</a>
      </div>
    </div>
  </section>

  <section class="section" id="lab">
    <div class="section-eyebrow">Hands-on lab</div>
    <h2>Scrub through the pipeline with live snippets</h2>
    <p class="lead">
      Toggle between compiler stages to see real commands and curated output from the snapshot suite.
      Copy, run, and iterate—each panel links to the golden sources in this repo.
    </p>
    <div class="lab">
      <div class="lab-tabs">
        <input type="radio" name="lab-stage" id="stage-lex" checked />
        <label for="stage-lex">Lex &amp; Parse</label>
        <div class="lab-panel">
          <h3>AST pretty-print from algebraic data types</h3>
          <p>Inspect syntax sugar removal while staying close to the original surface language.</p>
          <pre><code>$ cargo run --bin mica -- --ast --pretty examples/adt.mica
// Snippet excerpt
fn describe_animal(animal: Animal) -> String using Io {
    match animal {
        Animal::Cat(name) => format("Cat named {name}"),
        Animal::Dog { name, age } => format("Dog {name} aged {age}"),
    }
}</code></pre>
          <a class="sparkle-cta" href="https://github.com/mica-lang/mica/blob/main/examples/adt.mica">Open example ↗</a>
        </div>
        <input type="radio" name="lab-stage" id="stage-effects" />
        <label for="stage-effects">Effects &amp; Exhaustiveness</label>
        <div class="lab-panel">
          <h3>Capability and match diagnostics</h3>
          <p>See how `using` clauses gate IO and how the compiler flags non-exhaustive matches.</p>
          <pre><code>$ cargo run --bin mica -- --check examples/adt_match_nonexhaustive.mica
error[E0004]: non-exhaustive patterns: variants `Moth` and `Spider` not covered
  --> examples/adt_match_nonexhaustive.mica:27:9
   |
27 |         Insect::Ant(age) => describe_ant(age),
   |         ^^^^^^^^^^^^^^^ pattern `Insect::Moth(_)` and `Insect::Spider(_)` not covered</code></pre>
          <a class="sparkle-cta" href="https://github.com/mica-lang/mica/blob/main/examples/adt_match_nonexhaustive.mica">See diagnostics ↗</a>
        </div>
        <input type="radio" name="lab-stage" id="stage-ir" />
        <label for="stage-ir">Typed IR</label>
        <div class="lab-panel">
          <h3>Typed SSA lowering for methods</h3>
          <p>Observe lowered blocks and experiment with new passes before IR hands off to LLVM.</p>
          <pre><code>$ cargo run --bin mica -- --ir examples/methods.mica
function $0::main() -> () using {Io}
block0:
    %0 = load_const String("hello")
    %1 = call $0::greet(%0)
    return ()</code></pre>
          <a class="sparkle-cta" href="https://github.com/mica-lang/mica/blob/main/examples/methods.mica">Open IR source ↗</a>
        </div>
        <input type="radio" name="lab-stage" id="stage-runtime" />
        <label for="stage-runtime">Runtime Tracing</label>
        <div class="lab-panel">
          <h3>JSON traces for deterministic concurrency</h3>
          <p>Trace worker orchestration with structured JSON that powers docs and debugging.</p>
          <pre><code>$ cargo run --bin mica -- --run --trace-json - examples/concurrency_pipeline.mica
{
  "task": "root",
  "spawns": ["compress", "upload"],
  "events": [
    { "at": "0ms", "message": "compress-start" },
    { "at": "4ms", "message": "upload-ready" }
  ]
}</code></pre>
          <a class="sparkle-cta" href="https://github.com/mica-lang/mica/blob/main/examples/concurrency_pipeline.mica">Inspect trace ↗</a>
        </div>
      </div>
    </div>
  </section>

  <section class="section" id="pipeline">
    <h2>Follow the compiler pipeline end to end</h2>
    <p class="lead">
      Jump into any phase with an explicit CLI flag and compare the output to the golden files checked into
      the repository.
    </p>
    <div class="timeline">
      <div class="timeline-step">
        <h3>1. Lex & parse</h3>
        <p>Tokenise and pretty-print the AST to understand how syntax is desugared.</p>
        <code>cargo run --bin mica -- --ast --pretty examples/adt.mica</code>
      </div>
      <div class="timeline-step">
        <h3>2. Resolve & check effects</h3>
        <p>Enforce capability usage and match exhaustiveness with diagnostic snippets.</p>
        <code>cargo run --bin mica -- --check examples/adt_match_nonexhaustive.mica</code>
      </div>
      <div class="timeline-step">
        <h3>3. Lower to typed IR</h3>
        <p>Observe the typed SSA lowering and experiment with new passes.</p>
        <code>cargo run --bin mica -- --ir examples/methods.mica</code>
      </div>
      <div class="timeline-step">
        <h3>4. Emit native scaffolding</h3>
        <p>Inspect the LLVM-ready representation and generated C stubs.</p>
        <code>cargo run --bin mica -- --llvm examples/methods.mica</code>
      </div>
      <div class="timeline-step">
        <h3>5. Run with deterministic providers</h3>
        <p>Execute binaries with JSON traces that capture task lifecycles and capability usage.</p>
        <code>cargo run --bin mica -- --run --trace-json - examples/concurrency_pipeline.mica</code>
      </div>
    </div>
  </section>

  <section class="section" id="examples">
    <h2>Example gallery</h2>
    <p class="lead">Each example ships ready to run—copy the path, drop it into the CLI, and inspect the stages above.</p>
    <div class="example-grid">
      <div class="example-card">
        <h3>Concurrent pipeline orchestration</h3>
        <p>Structure fan-out/fan-in workflows with deterministic concurrency and auditable effects.</p>
        <footer><code>examples/concurrency_pipeline.mica</code></footer>
        <a href="https://github.com/mica-lang/mica/blob/main/examples/concurrency_pipeline.mica">Read the source ↗</a>
      </div>
      <div class="example-card">
        <h3>Effectful resource pools</h3>
        <p>Model scoped capability hand-offs with helper functions guarded by `using` clauses.</p>
        <footer><code>examples/effects_resource_pool.mica</code></footer>
        <a href="https://github.com/mica-lang/mica/blob/main/examples/effects_resource_pool.mica">Run the sample ↗</a>
      </div>
      <div class="example-card">
        <h3>Algebraic data types & pattern matching</h3>
        <p>Demonstrate exhaustive match diagnostics backed by snapshot tests.</p>
        <footer><code>examples/adt_match_nonexhaustive.mica</code></footer>
        <a href="https://github.com/mica-lang/mica/blob/main/examples/adt_match_nonexhaustive.mica">See the warnings ↗</a>
      </div>
      <div class="example-card">
        <h3>Native interop scaffolding</h3>
        <p>Inspect the generated C stubs and typed IR that bridge to the runtime.</p>
        <footer><code>docs/snippets.md</code></footer>
        <a href="https://github.com/mica-lang/mica/blob/main/docs/snippets.md">Walk through the snippets ↗</a>
      </div>
    </div>
  </section>

  <section class="section" id="resources">
    <h2>Deep dives & resources</h2>
    <div class="resource-grid">
      <div class="resource-card">
        <strong>Language tour</strong>
        <p>Concept-by-concept walkthrough paired with runnable programs.</p>
        <a href="https://github.com/mica-lang/mica/blob/main/docs/tour.md">Open docs/tour.md ↗</a>
      </div>
      <div class="resource-card">
        <strong>Module reference</strong>
        <p>Explore the compiler architecture across lexer, resolver, effect checker, and runtime orchestration.</p>
        <a href="https://github.com/mica-lang/mica/tree/main/docs/modules">Browse docs/modules ↗</a>
      </div>
      <div class="resource-card">
        <strong>Status dashboards</strong>
        <p>Review verification coverage, roadmap progress, and snapshot health.</p>
        <a href="https://github.com/mica-lang/mica/blob/main/docs/status_summary.md">Read status_summary.md ↗</a>
      </div>
      <div class="resource-card">
        <strong>Roadmap</strong>
        <p>Track upcoming milestones and areas where contributions are especially welcome.</p>
        <a href="https://github.com/mica-lang/mica/tree/main/docs/roadmap">See the roadmap ↗</a>
      </div>
    </div>
  </section>

  <section class="section" id="showcase">
    <div class="section-eyebrow">Showcase</div>
    <h2>A mosaic of compiler ergonomics</h2>
    <p class="lead">
      Mica’s toolchain is intentionally compact, so the repository doubles as a syllabus, a playground,
      and a launchpad for research ideas.
    </p>
    <div class="mosaic">
      <article class="mosaic-card">
        <strong>Teaching-first docs</strong>
        <p>Every tutorial is executed in CI, ensuring the copy you read matches the binary you run.</p>
        <footer>Snapshot verified • CI enforced</footer>
      </article>
      <article class="mosaic-card">
        <strong>Runtime instrumentation</strong>
        <p>Structured providers model IO, filesystems, and time for deterministic labs and demos.</p>
        <footer>Deterministic by design</footer>
      </article>
      <article class="mosaic-card">
        <strong>Ergonomic lowering</strong>
        <p>Typed SSA IR feels familiar to Rustaceans and leaves room for experimental passes.</p>
        <footer>Start hacking in an afternoon</footer>
      </article>
      <article class="mosaic-card">
        <strong>Composable effects</strong>
        <p>Capability rows make IO explicit while keeping ergonomics approachable for students.</p>
        <footer>Predictable control of side effects</footer>
      </article>
    </div>
    <div class="badge-strip">
      <span>Zero scaffolding runtime</span>
      <span>LLVM-ready backend</span>
      <span>CLI-first workflow</span>
      <span>Rust-powered compiler</span>
      <span>Friendly to contributors</span>
    </div>
  </section>

  <section class="section" id="contribute">
    <h2>Contribute, teach, and experiment</h2>
    <div class="highlight-block">
      <div>
        <strong>Ready to prototype a new effect system or teaching lab?</strong>
        Open an issue, share an idea in discussions, or spin up a PR with a runnable snippet.
      </div>
      <ul>
        <li>Discuss design questions in <a href="https://github.com/mica-lang/mica/discussions">GitHub Discussions</a>.</li>
        <li>File bugs or feature requests via <a href="https://github.com/mica-lang/mica/issues">Issues</a>.</li>
        <li>Keep docs honest by regenerating snapshots with <code>cargo run --bin gen_snippets -- --check</code>.</li>
      </ul>
    </div>
  </section>

  <p class="footer-note">Built with ❤️ by language and tooling enthusiasts. Read it in a weekend, extend it on Monday.</p>
</div>

<script>
  (function () {
    const demo = document.querySelector('.hero-demo[data-animated="true"]');
    if (!demo) return;
    const lines = Array.from(demo.querySelectorAll('.typing-line'));
    if (!lines.length) return;
    let index = 0;

    function typeLine(line) {
      const text = line.dataset.command || '';
      line.textContent = '';
      line.classList.add('active');
      let pos = 0;

      function tick() {
        line.textContent = text.slice(0, pos);
        if (pos <= text.length) {
          pos += 1;
          requestAnimationFrame(tick);
        } else {
          setTimeout(nextLine, 1600);
        }
      }

      requestAnimationFrame(tick);
    }

    function nextLine() {
      lines[index].classList.remove('active');
      index = (index + 1) % lines.length;
      lines.forEach((line, i) => {
        if (i !== index) {
          line.textContent = line.dataset.command || '';
        }
      });
      typeLine(lines[index]);
    }

    lines.forEach((line, i) => {
      line.textContent = line.dataset.command || '';
      line.classList.remove('active');
    });

    typeLine(lines[index]);
  })();
</script>
