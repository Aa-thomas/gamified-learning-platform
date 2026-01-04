
# Techdegree-Plus Project Outline — Week 1 Project 1: Deterministic Order Flow Simulator (30 pts)

## Module metadata

* **Module/Week:** Week 1 (Stage 2 kickoff / systems core)
* **Project:** Deterministic Order Flow Simulator
* **Language/Tools:** Rust 1.7x+, `clap` (CLI), `serde` (+ `toml`/`json` optional), `rand` (seeded), `criterion` (optional stretch), `insta` (optional)
* **Estimated time:** ~8–12 focused hours

---

## Hiring-signal claim (what this proves)

You can design a **small event-driven system** with:

* deterministic simulation + reproducible runs
* clean domain modeling (enums/traits/modules)
* invariants + tests that don’t depend on I/O
* “systems thinking” (metrics, regimes, throughput, stable iteration)

### Required evidence artifacts

* **README.md**: purpose, how to run, config flags, sample output/trace snippet, determinism proof (same seed), design diagram
* **DESIGN.md**: architecture, event loop, determinism strategy, agent model, invariants
* **TESTS**: unit tests for mechanics + metrics invariants; at least one “golden” deterministic trace test
* **RUNBOOK.md**: common failures, how to reproduce bugs (seed!), how to validate determinism
* *(Optional)* **BENCH.md**: events/sec baseline under fixed workload

---

## Learning objectives → measurable outcomes

1. **Event modeling & architecture**

   * Define `Event` + `State` + event loop with clear module boundaries.
2. **Determinism by construction**

   * Same `--seed` ⇒ identical trace + identical metrics (byte-for-byte if you serialize).
3. **Traits & polymorphism**

   * `Agent` trait with ≥2 implementations; agents generate intent/events without leaking randomness.
4. **Invariants + testing discipline**

   * Encode 5–10 invariants; test mechanics + metrics with pure core logic (no stdout/files).
5. **Operational polish**

   * CLI UX, structured output option, clear failure messages, reproducible runs.

---

## Required reading (exact targets you should use)

You already picked strong reads—here’s how to “aim” them at this build.

* **Rust in Action**

  * Ch. 2–3: focus on **enums + `match`**, ownership of event/state, modeling “messages”
* **Effective Rust**

  * Ch. 1 Items **1–3, 5, 9**: focus on **type-driven modeling**, `Option/Result` transforms, iterator correctness
* **Rust for Rustaceans**

  * Ch. 1: “real project” discipline (explicit invariants)
  * Ch. 6: test strategy + avoiding “accidental complexity” in tests

*(If you want one extra optional reading later: stable iteration order + hashing pitfalls—look up Rust’s `HashMap` non-deterministic iteration and when to prefer `BTreeMap`.)*

---

## System design (reference architecture)

### Core modules

* `domain/`

  * `event.rs`: `Event`, `EventKind`, IDs, timestamps/ticks
  * `order.rs`: `Order`, `Side`, `Symbol`, `Price`, `Qty`
  * `metrics.rs`: counters + derived rates + throughput
  * `regime.rs`: `Regime` + transition logic
* `sim/`

  * `engine.rs`: event loop + state machine
  * `agents.rs`: `Agent` trait + implementations
  * `rng.rs`: seeded RNG wrapper + deterministic helpers
* `io/` (shell)

  * `cli.rs`: args parsing
  * `output.rs`: text/json formatting (no logic)

### Dataflow

1. Seed → deterministic RNG
2. Tick loop
3. Agents propose actions → engine converts to events
4. Engine applies events → state update
5. Metrics updated + regime transitions
6. Optional trace sink (in-memory / file)

---

## Main spec

### Must (project requirements)

* **Events:** `NewOrder`, `Cancel`, `Fill` (simulated)
* **Metrics:** total orders/cancels/fills, cancel rate, queue depth, throughput (events/sec)
* **Regimes:** calm → burst → cancel-storm (time/threshold driven)
* **Mechanic (pick 1):**

  * burst scheduler **or**
  * cancel-storm agent **or**
  * deterministic risk/inventory limit **or**
  * toy matching rule with invariants
* **Agent abstraction:** `Agent` trait + ≥2 implementations (e.g., `NoiseTrader`, `CancelBot`)
* **Config:** CLI flags (minimum) and optional config file
* **Determinism:** `--seed` reproduces identical trace + identical summary

### Should (strongly recommended)

* **Stable ordering everywhere**

  * if you need maps: prefer `BTreeMap` / sorted `Vec`
* **Trace serializer**

  * `--output json` or `--trace file.rpl` that can be diffed
* **Golden test**

  * fixed seed ⇒ fixed first N events snapshot

### Could (stretch)

* `.rpl` replay + `--replay path`
* snapshot/load state
* criterion benchmark with fixed workload & seed

---

## Non-functional constraints (explicit engineering gates)

* **No panics in core** (`lib.rs` / `sim/` / `domain/`)

  * no `unwrap/expect` in library code
* **Deterministic iteration**

  * never rely on `HashMap` iteration order for output/trace
* **Pure-core testing**

  * core step function(s) must be testable without I/O
* **Observability**

  * errors are actionable; optional structured logs go to stderr
* **Performance sanity**

  * avoid O(n²) accidental loops in the hot path (document tradeoffs)

---

## Invariants (write them down in DESIGN.md and test them)

Pick ~8–12; here are good defaults:

* IDs are unique and monotonically assigned (or otherwise deterministic)
* `fills <= new_orders` (or justified if you model partial fills)
* cancel references an existing open order ID (or becomes `Reject/NoOp` deterministically)
* queue depth never negative; depth updates match event effects
* cancel rate in [0,1]
* regime transitions are deterministic and depend only on state/tick (not wall-clock)
* throughput computed from deterministic tick duration or measured wall-clock **but clearly separated** (don’t mix both)

---

## Micro-challenges (6–12) — each produces a proof artifact

1. **Deterministic IDs**

   * Goal: produce repeatable `OrderId` sequence across runs
   * Proof: unit test asserts first 5 IDs for seed X are fixed
2. **Stable event ordering**

   * Goal: guarantee `Vec<Event>` ordering is deterministic
   * Proof: golden test snapshot of first 20 events for seed X
3. **Regime transition function**

   * Goal: pure function `next_regime(state, tick) -> Regime`
   * Proof: table-driven tests for boundary thresholds
4. **Agent trait contract**

   * Goal: `Agent::step(&mut self, ctx: &Context) -> Vec<Action>`
   * Proof: compile-time enforcement + tests for two agents
5. **Metrics invariants**

   * Goal: `Metrics::apply(&Event)` never violates invariants
   * Proof: property-like tests over a fixed event script
6. **Mechanic correctness**

   * Goal: your chosen mechanic has 2–3 crisp invariants
   * Proof: targeted unit tests + one integration scenario
7. *(Optional)* **Replay serializer**

   * Goal: serialize events to JSON lines deterministically
   * Proof: output file diff identical on two runs with same seed
8. *(Optional)* **Engine vs shell separation**

   * Goal: core returns `Summary + Trace`, shell prints
   * Proof: tests never touch stdout

---

## Checkpoints (CP1–CP4) with “pass evidence”

### CP1 — Domain model + core step (foundation)

**Done when:**

* `Event`, `State`, `Agent` trait exist
* engine can run N ticks and produce metrics struct
  **Evidence:** unit tests for `apply_event` + `engine_step`

### CP2 — Determinism proof (seeded replay)

**Done when:**

* `--seed` wired end-to-end
* golden test: fixed seed ⇒ fixed first N events + metrics
  **Evidence:** snapshot test or asserted event list hash

### CP3 — Regimes + chosen mechanic integrated

**Done when:**

* calm/burst/cancel-storm transitions work
* mechanic changes event mix predictably
  **Evidence:** scenario test: regime progression + metric deltas

### CP4 — Polish + docs + ops

**Done when:**

* README sample run + design diagram
* RUNBOOK failure modes + reproduction steps
* clippy/rustfmt clean, CI green
  **Evidence:** `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`

---

## Deliverables + acceptance criteria

* **Crate:** `/sim/orderflow/`
* **Binary:** `cargo run -- --seed 42 --ticks 10_000 ...`
* **Docs:** `README.md`, `DESIGN.md`, `RUNBOOK.md`
* **Tests:** `cargo test` passes; includes at least:

  * mechanic tests
  * metrics invariants tests
  * determinism golden test
* **Design diagram:** ASCII/Mermaid embedded in README or DESIGN

---

## Rubric conversion (30 pts → 100-pt senior weighted)

* **Correctness & determinism** — 30
* **Design clarity (traits/enums/modules)** — 25
* **Testing & invariants** — 20
* **Robustness & error handling** — 15
* **Polish (docs/UX/ops)** — 10
  **Bonus (up to +10):** replay file + replay CLI, BENCH.md, exceptionally clean diagnostics

---

## Quiz pack (15–20 questions)

Tags: *(type: concept / code-read / scenario), difficulty: (E/M/H)*

1. Determinism: why `HashMap` iteration can break reproducibility *(concept, M)*
2. Seed plumbing: where randomness is allowed vs forbidden *(scenario, M)*
3. Design: `Event` enum vs trait objects—tradeoffs *(concept, M)*
4. Testing: golden test failure triage using `--seed` *(scenario, M)*
5. Invariants: pick 3 invariants for cancels/fills and justify *(concept, M)*
6. Error strategy: `Reject` event vs `Result::Err`—when each is better *(scenario, H)*
7. Stable time: tick-based time vs wall-clock time *(concept, E)*
8. Agent contract: what belongs in `Context` *(concept, M)*
9. Throughput: measuring events/sec without contaminating determinism *(scenario, H)*
10. Module boundaries: what goes in `domain` vs `sim` vs `io` *(concept, E)*
    11–18. Short code reading questions about `match`, ownership, iterator transforms *(code-read, E/M)*

---

## Mandatory interview talk track (6–8 minutes)

1. **What it is:** deterministic order-flow simulator + regimes + metrics
2. **Architecture:** domain vs engine vs agents vs shell
3. **Determinism strategy:** seeded RNG + stable ordering + golden tests
4. **Mechanic deep dive:** your chosen mechanic + invariants
5. **Testing:** pure-core tests + determinism snapshot
6. **Tradeoffs:** what you simplified (matching, pricing, latency realism) and why
7. **Next step:** replay file, snapshot/load, benchmark baseline

---

## Tooling / workflow requirements

* `cargo fmt --check`
* `cargo clippy -- -D warnings`
* `cargo test`
* Optional: `just` or `xtask` commands:

  * `just test`, `just lint`, `just run-seed 42`, `just replay path`
* CI: GitHub Actions running fmt/clippy/test

---

## Optional starter kit (folder skeleton)

```text
sim/orderflow/
  src/
    lib.rs
    domain/{mod.rs,event.rs,order.rs,metrics.rs,regime.rs}
    sim/{mod.rs,engine.rs,agents.rs,rng.rs}
    io/{mod.rs,cli.rs,output.rs}
  src/bin/orderflow.rs
  tests/determinism.rs
  README.md
  DESIGN.md
  RUNBOOK.md
```

---

## Final consistency checklist

* Objectives ↔ rubric ↔ deliverables all align
* Determinism proof exists (golden test + README instructions)
* No `unwrap/expect` in core
* Regime transitions are pure and test-covered
* Output ordering is stable (sorted / `BTreeMap`)
* README includes sample run + diagram
* RUNBOOK explains “how to reproduce” using seed + config

If you want, I can also generate the **Day-by-day (Mon–Fri) lecture + quiz + mini-challenges** for Week 1 using your “Pre-Project Prep Generator” format, but tailored to this simulator.
