# Ecosystem & Interop Strategy

The language succeeds only if developers can build real systems today and still feel ahead of the curve tomorrow. This plan
focuses on libraries, package distribution, and interoperability tracks that future-proof the ecosystem.

## Standard Library Growth

| Wave | Focus | Modules | Notes |
| --- | --- | --- | --- |
| S0 | Core primitives | `core::{Option, Result, List, Vec, Iterator}`, numeric traits, `String`, `Bytes`. | Stabilize APIs; ensure zero allocations in iterator adapters when possible. |
| S1 | Systems access | `sys::{fs, net, time, process}`, structured tasks, channel abstractions. | Respect capability boundaries; design async-friendly IO wrappers. |
| S2 | Data tooling | `data::{csv, json, arrow}`, streaming parsers, schema inference. | Provide iterators instead of full materialization; integrate with `mica::io` traits. |
| S3 | Numerics | `math::{vec, mat, linalg}`, BLAS-backed ops with safe fallbacks. | Expose GPU hooks via capability gating (`!{gpu}`). |
| S4 | Concurrency extras | `concurrent::{Mutex, RwLock, Actor}` for rare escape hatches. | Document determinism trade-offs and effect requirements. |

**Exit Criteria**
- Each wave has API docs, usage guides, and examples under `examples/`.
- Benchmarks show parity with equivalent Rust implementations within 10% for targeted workloads.

**Future-facing trajectory**
- Stage later waves (S5+) for privacy-preserving analytics, GPU/AI accelerators, and verifiable computation helpers.
- Publish stability levels so downstream teams can adopt modules with clear upgrade cadences.
- Link benchmarking harnesses to roadmap analytics for ongoing performance governance.

## Package Manager & Registry

1. Specify `mica.toml` manifest (targets, dependencies, capabilities) in `docs/specs/mica_toml.md`.
2. Build `mica package` CLI:
   - `mica package init`, `add`, `remove`, `update`.
   - Lockfile format with deterministic resolution.
3. Stand up hosted index prototype (S3 + CDN) with signed package manifests.
4. Document publishing workflow, including capability audits and lint gates.

**Exit Criteria**
- Sample workspace with two packages builds and runs via `mica build`.
- Publishing dry-run validates signatures and manifest schema.

**Future-facing trajectory**
- Support reproducible builds via content-addressed storage and optional attestations.
- Allow capability declarations in manifests to drive security reviews and automated policy checks.
- Explore decentralized mirrors with deterministic conflict resolution for global teams.

## Interoperability Tracks

1. **C ABI**: Provide `mica cbindgen` command to emit headers, plus runtime tests linking C callers.
2. **Python/JavaScript Foreign Tasks**:
   - Design message boundary protocol (JSON / Arrow) and isolation runtime.
   - Implement adapters in `crates/mica-foreign-{python,js}`.
   - Supply examples bridging to Pandas/Node streams.
3. **Rust & TypeScript Migration**:
   - Build transpiler prototypes for common subsets (iterators, ADTs).
   - Provide migration guides in `docs/migration/`.

**Exit Criteria**
- At least one end-to-end demo calling Python data processing from Mica with deterministic boundaries.
- Interop docs detail safety guarantees and limitations (no shared mutable state, explicit capabilities).

**Future-facing trajectory**
- Expand foreign-task adapters to structured AI workloads with auditable prompts/results.
- Provide schema-aware bridges (Arrow Flight, gRPC) with deterministic serialization strategies.
- Prototype dual-language debugging sessions where Mica and the host runtime share effect traces.

## Community & Governance

1. Publish contribution guidelines, RFC template, and governance document (`docs/GOVERNANCE.md`).
2. Establish RFC process: stage proposals, shepherds, decision logs.
3. Schedule community sync (monthly) and office hours (bi-weekly) once repo is public.

**Exit Criteria**
- First three RFCs (effect system, task runtime, package manager) processed end-to-end.
- Governance doc ratified by core team.

**Future-facing trajectory**
- Establish contributor ladders that recognize design, implementation, and stewardship equally.
- Automate RFC history linking into IDE hints (e.g., “this feature originated in RFC-12”).
- Build mentorship and residency programs to cultivate future compiler implementers.

---

_Track updates quarterly alongside roadmap reviews._
