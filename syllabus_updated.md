## Week 1 — Project 1: Deterministic Order Flow Simulator (30 pts)

### Learning Objectives
- Architect a small event-driven system: entities, state machines, event loop; traits & enums.
- Build deterministic replay mindset (same seed → same trace → same outcomes).

### Background
- Event loop anatomy; RNG seeding; stable iteration order; agent-based simulation.

### Reading (Mon–Tue)
- **Rust in Action**: Ch. 2 (*Language foundations*) + Ch. 3 (*Compound data types*) — loops/`match`/struct+enum modeling for events and agent state.  
- **Effective Rust**: Ch. 1 (*Types*) Items **1–3**, **5**, **9** — type-driven modeling + `Option/Result` transforms + iterator transforms for deterministic event loops.  
- **Rust for Rustaceans**: Ch. 1 (*Foundations*) + Ch. 6 (*Testing*) — “real project” habits: invariants, tests first, no hidden panics.

### Requirements (Wed–Thu)
- Build a CLI-driven simulator that generates and processes synthetic order-flow events:
  - Minimum events: `NewOrder`, `Cancel`, `Fill` (simulated).
  - Optional: `Amend`, `Reject` (e.g., risk limit).
- Replace “scoring + difficulty” with **metrics + regime progression**:
  - Metrics: total orders, cancels, fills, cancel rate, queue depth, event throughput (events/sec).
  - Regimes: calm → burst → cancel-storm (based on time/thresholds).
- One substantial mechanic (pick one):
  - Burst scheduler (rate spikes + decay).
  - Cancel-storm agent (adversarial cancels).
  - Simple risk/inventory limit (reject orders deterministically).
  - Toy matching rule (e.g., price-time simplified) with clear invariants.
- Clean entity abstraction (trait-based) and tidy simulation loop:
  - `Agent` trait with at least 2 implementations (e.g., `NoiseTrader`, `CancelBot`).
- Configurable controls:
  - CLI flags and/or config file for rates, regimes, symbols, and output format.
- Deterministic `--seed` option:
  - Same seed must reproduce identical event trace and identical summary metrics.

### Testing & Docs (Fri)
- Logic tests for your chosen mechanic and metrics invariants (pure core logic; no IO).
- README includes:
  - A sample run output (or short trace snippet).
  - A design diagram (ASCII/Mermaid/image).

### Stretch Goals
- Write a replay file (`.rpl`) containing the event trace.
- Snapshot/load minimal simulator state.

### Deliverables
- `/sim/orderflow/` crate, tests, `README.md`, design diagram.

### Rubric (30 pts)
- Feature depth & integration — 10  
- Design clarity (traits, enums, modules) — 8  
- Robustness (errors, edge cases, determinism) — 6  
- Polish (docs, UX, tests) — 6


## Week 2 — Lab 4: Order-Flow Simulator Hardening (25 pts)

### Learning Objectives
- Add complexity safely: time-step management, scheduling, and performance-aware updates.
- Practice profiling mindset and “budget” measurement without premature optimization.

### Background
- Fixed vs variable timestep; deterministic scheduling; bounded queues; simple performance counters.

### Reading (Mon–Tue)
- **Rust for Rustaceans**: Ch. 4 (*Error Handling*) + Ch. 6 (*Testing*) — typed errors, boundary design, and tests that lock in semantics under refactors.  
- **Effective Rust**: Ch. 3 (*Concepts*) Items **18** (*Don’t panic*) + **17** (*shared-state parallelism caution*) — hardening mindset: no panics in library code; keep failure explicit.  
- **Rust in Action**: Ch. 4 (*Lifetimes, ownership, and borrowing*) — clarifies the “why” behind borrow-checker pain you’ll hit while hardening.

### Requirements (Wed–Thu)
Extend **Project 1’s** simulator (same repo) with **two** of the following, and document the choice:

- **Regime scheduler:** scripted regime schedule (calm → burst → cancel-storm) with deterministic transitions.
- **Event-time simulation:** support event-time (timestamped) processing vs fixed ticks.
- **Overflow policy:** introduce a bounded queue and define what happens on overflow (drop oldest/newest, reject, etc.), with counters and logs.
- **Deterministic “jitter” injection:** apply seeded jitter/delay to specific event types for stress testing.
- **Gap/duplicate injection:** inject duplicates/out-of-order events and define a deterministic handling policy.

Add performance visibility:
- Record `events/sec`, queue depth, and per-tick duration (simple histogram or min/avg/max is fine).
- Add a CLI flag to print a compact performance summary at end of run.

### Testing & Docs (Fri)
- Unit tests for the new features (especially policies: overflow/gap/jitter).
- README update:
  - “Performance budget” section: what you measured and why.
  - Example run command showing the new flags/options.

### Stretch Goals
- Emit the event stream to a `.rpl` log (if not done in Week 4 stretch).
- Add a `--trace` mode that prints stable, reproducible trace lines for debugging.

### Deliverables
- Updated `/sim/orderflow/` crate, tests, updated `README.md`.

### Rubric (25 pts)
- Mechanic correctness (policies, scheduling) — 8  
- Performance awareness (counters, budget, reporting) — 5  
- Design clarity (modules, boundaries, readability) — 6  
- Tests & docs (coverage + clarity) — 6


---

## Week 3 — Project 2: Replayable Tape Viewer (CLI) (30 pts)

### Learning Objectives
- Build an operator-grade tool for **deterministic replay and inspection** of event streams.
- Reinforce the idea that **replay is the primary debugging primitive** in trading systems.
- Practice stable output design, filtering, and summary reporting.

### Background
- Real trading systems rely on **event logs (“tape”)** to reproduce bugs, analyze incidents, and validate invariants.
- A replay tool must be:
  - Deterministic (same input → same output),
  - Inspectable (filters, summaries),
  - Safe (no panics, robust to malformed input).

### Reading (Mon–Tue)
- **Rust in Action**: Ch. 7 (*Files and storage*) + (as needed) Ch. 9 (*Time and timekeeping*) — file I/O, buffering, timestamps.  
- **Rust for Rustaceans**: Ch. 5 (*Project Structure*) + Ch. 4 (*Error Handling*) — clean lib/bin separation for replay tools; ergonomic error surfaces.  
- **Effective Rust**: Ch. 4 (*Dependencies*) Items **21–26** — pulling in serde/clap/etc without API leakage or feature creep.

### Requirements (Wed–Thu)
- Build a CLI tool that **loads and replays** an event log produced by the simulator:
  - Input format: `.rpl` file (newline-delimited events or simple structured format).
  - Events must replay in **stable, deterministic order**.
- Core features:
  - **Replay controls**:
    - Full replay (default).
    - Optional step-by-step or rate-limited replay.
  - **Filtering**:
    - By event type (`NewOrder`, `Cancel`, `Fill`, etc.).
    - By symbol or agent (if present in events).
  - **Summaries**:
    - Total events by type.
    - Orders vs cancels vs fills.
    - Basic derived metrics (e.g., cancel rate).
- Output requirements:
  - Human-readable, line-oriented output suitable for golden testing.
  - Summary section printed at end of replay.
- Architectural constraints:
  - Parsing and replay logic must live in a **library crate**.
  - CLI must be a thin wrapper only.
  - No panics or `unwrap/expect` in library code.
  - Malformed events must return typed errors, not crash.

### Testing & Docs (Fri)
- Golden tests:
  - Fixed `.rpl` input → exact expected summary output.
- Unit tests for:
  - Filtering logic.
  - Summary aggregation.
- README includes:
  - Example replay command.
  - Example filtered replay.
  - “How to reproduce a bug from a trace” section.

### Stretch Goals
- Save/load replay state (cursor position + filters).
- Export summary report as JSON for downstream analysis.
- Optional TUI view (purely additive; CLI remains primary).

### Deliverables
- `/tools/tapeview/` (or `/crates/tape/`) crate.
- CLI binary.
- Tests, `README.md`, sample `.rpl` fixture.

### Rubric (30 pts)
- Determinism & correctness — 10  
- Tool usefulness & UX — 8  
- Error handling & robustness — 6  
- Tests & documentation — 6

---

## Week 4 — Lab: Order Lifecycle State Machine (25 pts)

### Learning Objectives
- Model **domain invariants explicitly** using enums and total pattern matching.
- Prevent illegal order transitions at compile time where possible.
- Build confidence reasoning about **correctness before performance**.

### Background
- In real trading systems, the majority of critical bugs are **state bugs**, not speed bugs.
- Orders must follow a strict lifecycle; invalid transitions must be impossible or explicitly rejected.
- Rust enums + exhaustive matching are ideal for encoding these guarantees.

### Reading (Mon–Tue)
- **Rust for Rustaceans**: Ch. 2 (*Types*) + Ch. 3 (*Designing Interfaces*) — model the order lifecycle as a *type-level* state machine with explicit transitions.  
- **Effective Rust**: Ch. 1 (*Types*) Items **1–2**, **6** (*newtype*) and Ch. 2 (*Traits*) Item **10** — encode invariants and reduce illegal states.  
- **Rust in Action**: revisit Ch. 3 (*Compound data types*) sections on enums + `match` when implementing transitions.

### Requirements (Wed–Thu)
- Implement a library crate modeling an **order lifecycle state machine**:
  - Required states:
    - `New`
    - `Pending`
    - `PartiallyFilled`
    - `Filled`
    - `Cancelled`
  - Required transitions (example, adjust if justified):
    - `New → Pending`
    - `Pending → PartiallyFilled | Filled | Cancelled`
    - `PartiallyFilled → PartiallyFilled | Filled | Cancelled`
- Encode transitions explicitly:
  - Use enums and methods like `apply(event)` or `transition(action)`.
  - Illegal transitions must:
    - Be impossible to express **or**
    - Return a typed error explaining why the transition is invalid.
- Track **transition reasons**:
  - e.g., cancel reason, fill quantity, rejection cause.
- Architectural constraints:
  - Library-only logic (no I/O).
  - No panics or `unwrap/expect`.
  - Exhaustive `match` statements (no wildcard arms hiding logic).

### Testing & Docs (Fri)
- Unit tests must prove:
  - All valid transitions succeed.
  - All invalid transitions fail deterministically with clear errors.
- Add a **state transition diagram** (ASCII or Mermaid) to the docs.
- Document invariants, e.g.:
  - “A `Filled` order can never transition again.”
  - “Filled quantity is monotonic.”

### Stretch Goals
- Add IOC/FOK semantics as constraints on transitions.
- Add property-style tests for random valid/invalid transition sequences.
- Integrate state machine into the simulator or order book from prior weeks.

### Deliverables
- `/orders/state/` (or `/crates/order_state/`) library crate.
- Unit tests.
- `README.md` with state diagram and invariants section.

### Rubric (25 pts)
- Transition correctness — 10  
- Invariant modeling & API design — 6  
- Test coverage & clarity — 5  
- Documentation & diagrams — 4

---

## Week 5 — Project: Mini Order Book + Matcher (Correctness First) (35 pts)

### Learning Objectives
- Implement a **price–time priority order book** with FIFO guarantees.
- Handle partial fills, cancellations, and edge cases correctly.
- Measure throughput and latency while prioritizing **correctness over speed**.

### Background
- The order book is the **canonical data structure** in trading systems.
- Most real-world failures come from subtle correctness bugs:
  - incorrect FIFO handling,
  - double fills,
  - missed cancels,
  - or broken invariants under edge cases.
- Performance matters, but only after correctness is provable.

### Reading (Mon–Tue)
- **Rust for Rustaceans**: Ch. 3 (*Designing Interfaces*) + Ch. 6 (*Testing*) — book APIs, invariants, and edge-case test suites (ties, partial fills, cancels).  
- **Effective Rust**: Ch. 2 (*Traits*) Item **12** (generics vs trait objects) + Ch. 1 Item **9** (iterator transforms) — design a matching API that stays fast *and* readable.  
- **Rust in Action**: Ch. 5 (*Data in depth*) — memory/layout intuition for hot paths (price levels, queues).

### Requirements (Wed–Thu)
- Implement a **single-symbol order book** as a library crate:
  - Separate bid and ask sides.
  - Price levels stored using an ordered map (`BTreeMap`).
  - Each price level maintains FIFO order (e.g., `VecDeque<Order>`).
- Supported operations:
  - Insert new order.
  - Cancel existing order.
  - Match incoming orders against the opposite side.
- Matching rules:
  - Price–time priority.
  - Partial fills supported.
  - Generate explicit trade events for each fill.
- Architectural constraints:
  - Core logic in a library crate only.
  - No panics or `unwrap/expect`.
  - All state mutations must preserve documented invariants.
- Benchmarks:
  - Criterion benchmarks for:
    - insert-only workload,
    - match-heavy workload.
  - Report throughput and basic latency statistics.

### Testing & Docs (Fri)
- Unit tests covering:
  - FIFO behavior at a single price level.
  - Partial fills across multiple orders.
  - Cancels before and after partial fills.
- Integration tests simulating short order sequences.
- Documentation must include:
  - Data structure choices and complexity analysis.
  - Explicit invariants (e.g., “order IDs are unique”, “no negative remaining quantity”).

### Stretch Goals
- Add support for multiple symbols.
- Implement IOC/FOK orders (state-machine driven).
- Emit a replayable event log compatible with the tape viewer.

### Deliverables
- `/book/mini/` (or `/crates/order_book/`) library crate.
- Criterion benchmarks.
- `README.md`, `DESIGN.md`, `BENCH.md`.

### Rubric (35 pts)
- Matching correctness & edge cases — 14  
- Data structure & invariant clarity — 8  
- Benchmark quality & methodology — 7  
- Tests & documentation — 6

--- 

## Week 6 — Lab: Concurrency Model Selection + API Refactor (25 pts)

### Learning Objectives
- Select and justify a **concurrency model** for the order book.
- Refactor APIs for clarity, safety, and long-term maintainability.
- Practice explaining **trade-offs**, not just implementations.

### Background
- There is no single “best” concurrency model for trading systems.
- Senior engineers are judged on:
  - why a model was chosen,
  - what it optimizes for,
  - and what it explicitly does *not* optimize for.
- Clean APIs and explicit boundaries reduce future performance and correctness risks.

### Reading (Mon–Tue)
- **Rust Atomics & Locks**: Ch. **1–3** (*Basics*, *Atomics*, *Memory Ordering*) — the mental model for choosing “actor/channels vs locks vs single-writer” correctly.  
- **Rust for Rustaceans**: Ch. 10 (*Concurrency and Parallelism*) — practical patterns for safe concurrency in real code.  
- **Effective Rust**: Ch. 3 Item **17** (*Be wary of shared-state parallelism*) — forces you to justify your model in DESIGN.md.

### Requirements (Wed–Thu)
- Choose **one** concurrency model for the order book:
  - Single-writer with multi-reader access.
  - Sharded locks by price or side.
  - Actor / channel-based serialization.
- Implement the chosen model:
  - Document lock boundaries or message flow explicitly.
  - Ensure all invariants from Week 7 remain valid under concurrency.
- API refactor:
  - Reduce public surface area.
  - Improve naming and type aliases.
  - Add doc comments and examples for public APIs.
- Architectural constraints:
  - Library code remains panic-free.
  - Concurrency primitives must be encapsulated (no leaking locks to callers).

### Testing & Docs (Fri)
- Concurrency-focused tests:
  - Concurrent inserts and cancels.
  - Concurrent reads during matching (if applicable).
- Update documentation:
  - Concurrency rationale section in `DESIGN.md`.
  - Explicit list of trade-offs and rejected alternatives.

### Stretch Goals
- Compare against an alternative concurrency model in a short appendix.
- Add lightweight contention metrics (e.g., lock hold time counters).

### Deliverables
- Refactored order book crate.
- Updated `DESIGN.md` with concurrency rationale.
- Updated benchmarks and tests.

### Rubric (25 pts)
- Concurrency correctness — 10  
- Quality of trade-off analysis — 6  
- API clarity & ergonomics — 5  
- Tests & documentation — 4  


---

## Week 7  — Lab: Observability + Performance Baseline (“Quant Top”) (25 pts)

### Learning Objectives
- Add **production-style observability** (metrics + structured logs) to the trading system.
- Establish a repeatable **performance baseline** with latency percentiles.
- Practice measuring the system without benchmarking the “shell” (CLI/I/O).

### Background
- A trading system without observability is not debuggable in production.
- Percentiles (p50/p95/p99) matter more than averages.
- You cannot optimize what you cannot measure — and you must measure **deterministically**.

### Reading (Mon–Tue)
- **Zero to Production in Rust**: Ch. 1 (*Getting Started* — CI/clippy/formatting/testing pipeline) + Ch. 4 (*Telemetry*) — production-grade observability habits (structured logs/tracing/metrics).  
- **Effective Rust**: Ch. 5 (*Tooling*) Items **29–32** — clippy discipline, “more than unit tests,” tooling ecosystem, CI.  
- **Rust for Rustaceans**: Ch. 13 (*The Rust Ecosystem*) — docs.rs, features, cargo workflows you’ll rely on for profiling + packaging.

### Requirements (Wed–Thu)
- Implement an observability module/crate:
  - Structured logs (event counts, queue depth, error categories).
  - Metrics:
    - throughput (events/sec or ops/sec)
    - p50/p95/p99 latency for core operations (match/insert/cancel)
    - allocation counts (at least coarse: allocs/op or bytes/op if available)
    - basic process stats where feasible (RSS/CPU% optional)
- Provide a **metrics endpoint** (text format is fine) OR a `--metrics` report mode:
  - Must be fast enough to run continuously without dominating runtime.
- Add a repeatable performance harness:
  - Deterministic workload generator (seeded).
  - Measures core library operations, not printing/parsing.

### Testing & Docs (Fri)
- Demonstrate measurement stability:
  - Run baseline 5 times; report variance (<10% recommended).
- Provide:
  - A short flamegraph or perf note showing top hot-path functions.
  - A baseline table (throughput + p50/p95/p99) in `EVAL.md`.

### Stretch Goals
- Prometheus-compatible text output.
- Histograms for latency buckets.
- `tracing` spans around major pipeline sections.

### Deliverables
- `/obs/` crate/module (metrics + logs).
- `/bench/` or harness crate for repeatable workloads.
- `EVAL.md` baseline performance report.
- Updated `RUNBOOK.md` with “how to measure” section.

### Rubric (25 pts)
- Correctness of metrics & logging — 10  
- Measurement methodology & repeatability — 8  
- Design quality (low overhead, good boundaries) — 5  
- Documentation & usability — 2  

## Week 8 — Lab: Durable Logging (WAL) + Snapshot + Crash Recovery (30 pts)

### Learning Objectives
- Implement durability mechanisms so **no committed events are lost**.
- Build a crash-recovery story using **WAL replay + snapshots**.
- Practice failure-mode thinking and deterministic recovery validation.

### Background
- Trading systems often treat the event log as the source of truth.
- Crash consistency is a systems skill: correctness requires explicit rules for
  what “committed” means and how recovery reconstructs state.
- WALs must be verifiable (checksums) and replayable deterministically.

### Reading (Mon–Tue)
- **Rust in Action**: Ch. 7 (*Files and storage*) — buffering, `fsync` realities, and file APIs you’ll touch in WAL/snapshots.  
- **Rust for Rustaceans**: Ch. 4 (*Error Handling*) + Ch. 5 (*Project Structure*) — “engine vs shell” boundaries so recovery logic stays testable.  
- **Zero to Production in Rust**: Ch. 8 (*Error Handling*) — think in failure modes and operator-facing diagnostics.

### Requirements (Wed–Thu)
- Implement a persistence crate/module that supports:
  - Append-only **WAL** for order-book events (or trades/fills + cancels).
  - Integrity checks:
    - per-record checksum (CRC32 or similar)
    - detectable corruption (typed `CorruptData` error)
  - Periodic **snapshot** of reconstructed state (book state or derived state).
  - Replay tool or library entrypoint:
    - load snapshot
    - replay WAL tail
    - rebuild identical final state deterministically
- Define explicit semantics:
  - What is “committed”?
  - What happens if last record is partial/corrupt?
  - What gets rejected vs recovered?

### Testing & Docs (Fri)
- Crash recovery tests:
  - Generate events → persist → simulate “kill” → recover → compare state.
  - Random corruption test:
    - corrupt a WAL record; ensure corruption is detected and handled deterministically.
- Document in `RUNBOOK.md`:
  - Recovery steps
  - How to diagnose corruption
  - Expected operator actions

### Stretch Goals
- Faster restart via snapshot frequency tuning; measure restart time.
- WAL compaction policy.
- Deterministic “kill test” harness (random kill points under a fixed seed).

### Deliverables
- `/persistence/wal/` crate/module + replay tool.
- Tests (unit + integration).
- `DESIGN.md` update: durability model + invariants.
- `RUNBOOK.md` update: failure modes + recovery procedure.

### Rubric (30 pts)
- Correctness of WAL + replay — 12  
- Robustness (corruption detection, edge handling) — 7  
- Design clarity (semantics + invariants) — 6  
- Tests & documentation — 5

---

## Week 9 — Lab: Failure Injection + Invariant Monitoring + SLOs (30 pts)

### Learning Objectives
- Prove resilience by running the system under **controlled failure scenarios**.
- Define and enforce **invariants** (no duplicate fills, monotonic sequences, consistent state).
- Package lightweight **SLOs + runbooks** that make the system operable.

### Background
- “Works on my machine” is not a systems outcome.
- Production readiness is the ability to:
  - detect failures quickly,
  - preserve invariants under stress,
  - recover deterministically,
  - and explain what happened with evidence.
- In trading systems, correctness invariants matter more than uptime.

### Reading (Mon–Tue)
- **Rust for Rustaceans**: Ch. 6 (*Testing*) + Ch. 4 (*Error Handling*) — failure injection is just *tests with teeth*; errors must be observable and actionable.  
- **Zero to Production in Rust**: Ch. 6 (*Domain modelling / invariants*) + Ch. 8 (*Error Handling*) — enforce invariants with types; make failures legible for operators.  
- **Effective Rust**: Ch. 5 Item **30** (*Write more than unit tests*) + Ch. 3 Item **18** (*Don’t panic*).

### Requirements (Wed–Thu)
- Implement a **failure injection harness** (CLI flags or test harness) that can deterministically trigger:
  - WAL corruption in a specific segment/record.
  - Process kill at a deterministic point (simulate crash).
  - Artificial latency spike (sleep forbidden in production paths; inject via test clock or harness hooks).
  - Dropped events / out-of-order events (if your system models sequencing).
- Define invariants and wire them into the system:
  - Create `INVARIANTS.md` that lists 8–12 invariants (examples):
    - No duplicate fills for the same order ID + fill sequence.
    - Remaining quantity is never negative.
    - Filled quantity is monotonic.
    - Replay produces identical final state given same snapshot+WAL.
    - Event sequence numbers are monotonic within a stream.
  - Implement invariant checks:
    - As assertions in tests (preferred).
    - As runtime counters/alerts in observability module (no panics).
- Introduce lightweight SLOs:
  - Example targets (adjust to your system):
    - p95 match latency < X ms under baseline load.
    - Recovery time < Y seconds from snapshot+WAL.
    - Zero invariant violations during soak run.

### Testing & Docs (Fri)
- Run a **30–60 minute soak** under deterministic failure scenarios:
  - Produce a run log and summary.
  - Confirm invariants remain true (or document discovered bug + fix).
- Update documentation:
  - `RUNBOOK.md` must include:
    - How to run the failure harness.
    - How to interpret invariant counters/logs.
    - “Recovery checklist” for common failures.
  - `SLOs.md` includes your chosen targets and why.

### Stretch Goals
- Add a “chaos profile” that cycles failures on a deterministic schedule.
- Persist invariant-violation samples for debugging (without crashing).

### Deliverables
- Failure injection harness (CLI mode or test harness).
- `INVARIANTS.md`, `SLOs.md`, updated `RUNBOOK.md`.
- Soak run logs / summary report (in `docs/` or `EVAL.md`).

### Rubric (30 pts)
- Failure coverage & deterministic injection — 10  
- Invariant quality & enforcement — 10  
- SLO definitions & operational maturity — 6  
- Documentation & evidence (soak logs, runbook clarity) — 4  

---


## Week 10 — Lab: Flagship Optimization (Cache Layout OR Lock-Free Queue) (30 pts)

### Learning Objectives
- Deliver one **measurable** low-latency improvement with evidence.
- Practice performance engineering:
  - hypothesis → measurement → change → verification.
- Demonstrate sound judgment about unsafe boundaries or data-oriented design.

### Background
- Optimization without a baseline is guessing.
- The goal is not “make it fast” — it’s “make one part faster and prove it.”
- Senior performance work always includes:
  - measurement hygiene,
  - trade-off documentation,
  - and regression prevention.

### Reading (Mon–Tue)
- **Rust Atomics & Locks**: Ch. 3 (*Memory Ordering*) + (pick one track) Ch. 5 (*Channels*) **or** Ch. 4 (*Spin Lock*) — if you’re doing lock-free/queue work, you need the real model.  
- **Rust in Action**: Ch. 6 (*Memory*) + Ch. 5 (*Data in depth*) — cache/alloc intuition for layout experiments.  
- **Effective Rust**: Ch. 3 Item **20** (*Avoid the temptation to over-optimize*) — guardrail: benchmark → change → re-measure.

### Requirements (Wed–Thu)
Choose **one** flagship track and complete it fully:

#### Track A — Cache-Friendly Order Book Layout
- Identify your hottest path via flamegraph/perf notes (from Week 9).
- Refactor a hot structure for locality:
  - reduce pointer chasing,
  - avoid redundant allocations,
  - consider AoS → SoA where it makes sense,
  - tighten memory layout (alignment / packing decisions documented).
- Maintain correctness invariants and deterministic behavior.

#### Track B — Lock-Free / Low-Contention Queue Between Pipeline Stages
- Implement or integrate a bounded queue for one high-traffic boundary (example: ingest → matcher).
- Requirements:
  - bounded capacity
  - defined overflow policy
  - safe external API
  - if `unsafe` is used internally: document invariants and keep unsafe blocks minimal
- Compare against a baseline (mutex/channel) fairly.

#### Evidence requirements (both tracks)
- Benchmark before and after:
  - throughput
  - p50/p95/p99 latency
- Include methodology:
  - fixed workload seed
  - number of runs
  - how variance was handled

### Testing & Docs (Fri)
- Regression tests:
  - invariants still hold
  - determinism still holds
- `BENCH.md` must include:
  - before/after tables
  - short interpretation (“why did this win/lose?”)
  - what you would try next (even if not implemented)

### Stretch Goals
- Add allocator-level instrumentation (bytes/op trend).
- Add perf counter notes (branch misses / cache misses) if available on your platform.

### Deliverables
- Completed Track A or Track B changes.
- Updated benchmarks and `BENCH.md` (before/after + methodology).
- Updated `DESIGN.md` (trade-offs and constraints).
- Updated flamegraph/perf notes.

### Rubric (30 pts)
- Evidence-based improvement (measured, repeatable) — 12  
- Engineering judgment & trade-off clarity — 8  
- Correctness preserved (invariants, determinism) — 6  
- Documentation & regression prevention — 4  

## Week 11 — Deterministic Backtesting Engine + Strategy Interface (30 pts)

### Learning Objectives
- Build a fully **deterministic backtesting engine** that separates simulation time from wall-clock time.
- Design a clean **strategy interface** that supports repeatable evaluation.
- Produce reproducible fills and PnL suitable for research iteration.

### Background
- In quant research, determinism is not optional:
  - Same input + same configuration + same seed must produce identical outputs.
- A backtester is a systems program:
  - event loop design,
  - time modeling,
  - correctness invariants,
  - and reproducible evaluation artifacts.

### Reading (Mon–Tue)
- **Rust for Rustaceans**: Ch. 3 (*Designing Interfaces*) + Ch. 6 (*Testing*) — strategy traits, clean boundaries, deterministic tests.  
- **Effective Rust**: Ch. 2 Item **12** (generics vs trait objects) + Item **10** (standard traits) — design your strategy interface intentionally.  
- **Rust in Action**: Ch. 9 (*Time and timekeeping*) — time-series handling, timestamps, and simulation clocks.

### Requirements (Wed–Thu)
- Implement a library crate providing a deterministic backtest loop:
  - Explicit simulation clock (no wall-clock in the core).
  - Seeded RNG for any stochastic elements (if used).
  - Input stream:
    - synthetic events from your simulator **or**
    - replay logs from the tape viewer.
- Define a `Strategy` abstraction:
  - Trait-based strategy interface (e.g., `on_event`, `on_fill`, `on_bar`).
  - At least **2 strategies** (simple is fine; correctness > sophistication), e.g.:
    - momentum-ish (trend follow toy),
    - mean reversion toy,
    - or market-making toy.
- Outputs (must be deterministic):
  - trade/fill stream
  - end-of-run PnL summary
  - basic risk stats (e.g., max drawdown OR exposure over time)

### Testing & Docs (Fri)
- Determinism tests:
  - Same inputs + seed ⇒ identical trace hash and identical final PnL.
- Unit tests for:
  - strategy callbacks and state handling
  - PnL accounting invariants (no negative cash drift without trades, etc.)
- Documentation includes:
  - Backtest assumptions (fill model stub if needed).
  - How to run a backtest from a saved trace.

### Stretch Goals
- Add a plugin registry or config-driven strategy selection.
- Add “regime runs” (same strategy across multiple simulated regimes) with a comparison table.

### Deliverables
- `/backtest/engine/` crate (library).
- Backtest CLI wrapper.
- `DESIGN.md` update: determinism guarantees and evaluation outputs.
- `EVAL.md` with at least one strategy comparison table.

### Rubric (30 pts)
- Determinism correctness — 12  
- Architecture clarity (clock, loop, boundaries) — 8  
- Accounting correctness (PnL + stats) — 6  
- Docs/tests — 4

## Week 12 — Execution Simulation + Latency / Slippage Modeling (30 pts)

### Learning Objectives
- Model realistic execution behavior beyond “instant fills.”
- Add **latency, fees, and slippage** and quantify their impact on strategy results.
- Produce evidence that the evaluation pipeline is execution-aware.

### Background
- Most strategies look good under unrealistic fills.
- Adding latency and slippage is a key step from “toy backtest” → “research tool.”
- Execution modeling must remain deterministic:
  - sampled latency must be seeded,
  - and fills must be reproducible.

### Reading (Mon–Tue)
- **Rust in Action**: Ch. 9 (*Time and timekeeping*) + (as needed) Ch. 10 (*Processes, threads, and containers*) — latency, timers, and controlled scheduling.  
- **Rust Atomics & Locks**: Ch. 2 (*Atomics*) sections on counters/flags + Ch. 1 refresh — useful for latency measurement scaffolding that doesn’t distort results.  
- **Rust for Rustaceans**: Ch. 10 (*Concurrency and Parallelism*) — keep execution simulation scalable without turning it nondeterministic.

### Requirements (Wed–Thu)
- Implement an execution simulation module/crate integrated into the backtester:
  - Latency injection:
    - fixed delay model, and
    - one distribution-based model (seeded).
  - Fees model (simple schedule is fine).
  - Slippage model (choose one and document it):
    - fixed bps per trade,
    - volatility-linked bps,
    - or spread-based slippage derived from your simulated book.
- Execution behaviors (minimum):
  - partial fills supported (even if simplified)
  - reject scenarios supported (e.g., insufficient liquidity, invalid order, risk reject hook)
- Evaluation requirement:
  - Run the same strategy twice:
    - “ideal fills”
    - “latency + slippage + fees”
  - Compare PnL and at least 2 secondary metrics (e.g., win rate, turnover, drawdown).

### Testing & Docs (Fri)
- Tests:
  - latency model determinism (same seed → same sequence)
  - slippage/fee correctness (accounting checks)
  - partial fill edge cases
- Documentation:
  - `EVAL.md` must include a table showing impact of execution realism.
  - `DESIGN.md` update: execution model assumptions and limitations.

### Stretch Goals
- Support IOC/FOK semantics via your order state machine integration.
- Add queue-position / priority approximation (documented as an approximation).
- Add a “latency stress sweep” run (vary latency and report PnL degradation).

### Deliverables
- `/execution/venue_sim/` (or `/execsim/`) crate/module.
- Updated backtester wiring.
- `EVAL.md` with before/after execution realism comparison.
- Updated tests and docs.

### Rubric (30 pts)
- Modeling correctness (latency/fees/slippage) — 12  
- Determinism preserved — 6  
- Evaluation quality (clear comparison + interpretation) — 8  
- Tests & docs — 4  

## Week 13 — Lab 32: Risk Engine (Pre-Trade + Runtime Limits) (30 pts)

### Learning Objectives
- Implement **risk controls** that enforce invariants under load.
- Separate “risk decision” from “execution/matching” so the core loop stays predictable.
- Practice building denial paths that are deterministic, tested, and explainable.

### Background
- Risk is what makes a trading system “real.”
- Many systems fail not because they cannot trade, but because they cannot **safely refuse to trade**.
- Risk must be:
  - low-latency,
  - deterministic,
  - and correct under concurrency.

### Reading (Mon–Tue)
- **Zero to Production in Rust**: Ch. 6 (*Domain modelling / invariants*) + Ch. 8 (*Error Handling*) — risk limits are *types + policy*, and failures must be explicit.  
- **Effective Rust**: Ch. 1 Items **1–2**, **6** (newtype) — encode limits (qty/notional/leverage) as domain types, not loose numbers.  
- **Rust for Rustaceans**: Ch. 2 (*Types*) + Ch. 4 (*Error Handling*) — keep the risk engine API hard to misuse.

### Requirements (Wed–Thu)
- Implement a risk module/crate integrated into your execution/backtest pipeline:
  - Pre-trade checks (minimum):
    - max order size
    - max notional
    - position limit
  - Runtime controls (minimum):
    - drawdown limit OR loss limit
    - kill switch (manual or rule-based)
- Risk decision API:
  - Returns a typed allow/deny outcome (no panics).
  - Denials must include a reason code suitable for logging and analysis.
- Concurrency/architecture constraint:
  - Risk checks must not stall the matching/execution path unnecessarily.
  - Document your chosen approach:
    - single-threaded risk in the event loop,
    - sharded state,
    - or message/actor separation.
- Determinism constraint:
  - Same inputs + same seed/config ⇒ identical risk decisions and outcomes.

### Testing & Docs (Fri)
- Unit tests must cover:
  - each rule triggers correctly at the boundary and just beyond it
  - denial reason codes are correct and stable
- Integration tests:
  - strategy run that exceeds limits and is deterministically denied
  - restart/replay produces identical risk outcomes
- Documentation:
  - `DESIGN.md` update: risk model, invariants, and trade-offs
  - `RUNBOOK.md` update: how to configure limits, interpret denials, and trigger kill switch

### Stretch Goals
- Add per-strategy risk budgets (allocation) and throttling.
- Add latency measurement for risk checks and a p95 target.
- Add “risk incident” write-up (what failed, how it was detected, what action is taken).

### Deliverables
- `/risk/` crate/module.
- Updated backtest/execution wiring.
- Updated `DESIGN.md`, `RUNBOOK.md`.
- Tests demonstrating deterministic denials.

### Rubric (30 pts)
- Risk correctness (rules, boundaries, denial reasons) — 14  
- Robustness under load / concurrency story — 8  
- Determinism preserved — 4  
- Docs/tests quality — 4  

## Week 14 — Capstone: Paper Trading Mode + Portfolio Release (50 pts)

### Learning Objectives
- Integrate the full repo into a **usable, reproducible trading system prototype**.
- Demonstrate senior engineering maturity:
  - deterministic replay,
  - measurable performance,
  - durability and recovery,
  - operational docs and failure procedures.
- Package the work so a hiring manager can evaluate it quickly and confidently.

### Background
- The final hiring signal is not “I built many projects.”
- The final hiring signal is: **one coherent system** with boundaries, invariants, evidence, and a clear story.
- This week converts your repo from “a collection of components” into a **portfolio-grade system**.

### Reading (Mon–Tue)
- **Effective Rust**: Ch. 5 Items **27**, **31–32** — docs, tooling ecosystem, CI that prevents regressions during the portfolio release sprint.  
- **Zero to Production in Rust**: revisit Ch. 4 (*Telemetry*) + Ch. 1 CI sections — “ship it” discipline: logs/metrics + automated checks.  
- **Rust for Rustaceans**: Ch. 5 (*Project Structure*) + Ch. 13 (*Ecosystem*) — packaging, workspace hygiene, and release readiness.

### Requirements (Wed–Thu)
- Implement **paper trading mode** (“replay-as-live”):
  - Consume a trace stream (file-based replay is fine).
  - Run strategy decisions through execution simulation + risk.
  - Emit live-updating summaries (CLI is sufficient; TUI optional).
- End-to-end system requirements:
  - **Determinism mode:** same input + same config + same seed ⇒ identical outcomes (trace hash + PnL summary).
  - **Durability mode:** WAL + snapshot recovery works end-to-end (restore identical final state).
  - **Observability mode:** metrics + structured logs are enabled and documented.
- Provide a deterministic demo script:
  - One command to run a full scenario:
    - load/generate trace → replay-as-live → produce final summary + artifacts.
  - Output should be stable enough for golden testing.

### Testing & Docs (Fri)
- Quality gates (must pass from a clean clone):
  - `cargo test` (unit + integration) is green.
  - `cargo clippy -D warnings` is green.
  - `cargo fmt` is clean.
  - Bench harness runs and reproduces baseline results within documented variance.
- Portfolio artifact set (must all exist and agree with each other):
  - `README.md` (what it is, quickstart, demo commands, expected outputs)
  - `DESIGN.md` (architecture diagram + invariants + concurrency model rationale)
  - `BENCH.md` (methodology + results + regressions + before/after if applicable)
  - `EVAL.md` (strategy results + execution realism impact table)
  - `RUNBOOK.md` (recovery steps, failure injection, debugging workflow)
  - `INVARIANTS.md` (final invariant list + where enforced/tests)
  - `DECISIONS.md` (key trade-offs + rejected alternatives)
- Include at least one “senior narrative” write-up:
  - One correctness bug found via replay/failure injection and how it was fixed **or**
  - One performance bottleneck found and improved with evidence.

### Stretch Goals
- Add a lightweight dashboard snapshot (optional): export metrics to a file and include screenshots.
- Add a “replay pack” folder with 3 curated traces:
  - calm baseline
  - burst
  - cancel-storm

### Deliverables
- One-repo release tag (e.g., `v1.0`).
- Demo script + sample traces.
- Full documentation set: `README.md`, `DESIGN.md`, `BENCH.md`, `EVAL.md`, `RUNBOOK.md`, `INVARIANTS.md`, `DECISIONS.md`.
- Bench + evaluation outputs committed (or referenced reproducibly with exact commands).

### Rubric (50 pts)
- End-to-end correctness & determinism — 18  
- Architecture quality & trade-off clarity — 12  
- Performance evidence (benches + reasoning) — 8  
- Reliability & recovery story (WAL/snapshot + runbook) — 6  
- Documentation, polish, and usability — 6  

## Crypto Live Execution v0.1 (6 Weeks) — Multi-Venue Spot / Perpetuals Trading + Operator Console

> **Goal:** Build and run a **production-grade live crypto trading system** for a 2-person quant team:
> - Real-time market data via exchange WebSocket + REST
> - Direct exchange execution (no broker abstraction)
> - Deterministic recording, replay, and reconciliation
> - A thin **Neo.mjs operator console** for observability and safe control
> - Strict **fail-closed** behavior (if uncertain → stop trading)
>
> **Non-negotiables (entire stage)**
> - No `unwrap/expect` in library crates
> - Deterministic recording + replay of *all* market, decision, and exchange events
> - Continuous reconciliation vs exchange REST truth
> - Live trading requires explicit **`--armed`** mode + hard risk caps
> - If order book, positions, or balances are uncertain → **halt**
> - UI is **observer-only** and never a source of truth

---

## Week 1 — Live Crypto Market Data Ingestion + Normalization (25 pts)

### Learning Objectives
- Build a **robust exchange data adapter** for WebSocket + REST.
- Normalize heterogeneous exchange messages into a unified internal schema.
- Detect and respond to **data uncertainty** (disconnects, gaps, stale feeds).
- Expose normalized market data and health signals to a presentation layer.

### Background
- Crypto market data is unreliable by default:
  - dropped messages
  - reordered updates
  - partial snapshots
- A production system must *assume failure* and recover deterministically.
- Presentation layers must tolerate reconnects and partial visibility.

### Reading (Mon–Tue)
- OSTEP: I/O and failure modes (review).
- *Effective Rust*: boundary design and error modeling.
- Rust Book Ch. 8 (Collections, ordering guarantees).

### Requirements (Wed–Thu)
- Implement a crypto market-data ingestion module:
  - WebSocket stream for trades + top-of-book or depth.
  - REST snapshot bootstrap on startup and reconnect.
  - Exponential backoff reconnect with jitter.
- Normalization:
  - Canonical instrument ID (`exchange:symbol`).
  - Unified event types (Trade, BookUpdate, Snapshot).
- Data health:
  - sequence/gap detection (if provided by venue).
  - heartbeat-based staleness detection otherwise.
  - “data uncertain” flag propagated to risk layer.
- Backpressure:
  - bounded buffers with documented drop or stall policy.
- Metrics stream:
  - expose market health + throughput for UI consumption
  - snapshot + coalesced delta policy documented.

### Testing & Docs (Fri)
- Unit tests for:
  - message normalization
  - reconnect + resubscribe behavior
- Golden test:
  - recorded exchange stream → stable normalized output.
- `RUNBOOK.md`: “Market data stale / desync procedure.”

### Deliverables
- `/crates/crypto_md/`
- metrics stream endpoint (WS or SSE)
- `DESIGN.md` update: ingestion, recovery, and UI feed policy
- `RUNBOOK.md` update

### Rubric (25 pts)
- Ingestion robustness — 10  
- Normalization correctness — 7  
- Determinism & tests — 5  
- Documentation — 3  

---

## Week 2 — Instrument Model + Position & Margin Accounting (25 pts)

### Learning Objectives
- Model **spot and perpetual instruments** cleanly.
- Track positions, entry price, unrealized PnL, and margin usage.
- Understand liquidation mechanics at a systems level.
- Surface portfolio state safely to a presentation layer.

### Background
- Crypto risk is dominated by:
  - leverage
  - liquidation thresholds
  - funding rate bleed
- Incorrect accounting causes silent blowups.
- UI displays state; it does not compute it.

### Reading (Mon–Tue)
- Chan: sections on leverage and risk (conceptual).
- Rust Book Ch. 10 (traits + generics).

### Requirements (Wed–Thu)
- Implement instrument models:
  - Spot
  - Perpetual futures (mark price vs last price).
- Position accounting:
  - size, avg entry, realized/unrealized PnL.
  - leverage and margin usage.
- Liquidation modeling (approximate but explicit):
  - liquidation price calculation
  - liquidation buffer metric.
- Invariants:
  - no negative balances
  - position sign consistency
  - monotonic timestamps per instrument.
- Portfolio snapshot output:
  - read-only summary suitable for UI display.

### Testing & Docs (Fri)
- Unit tests for:
  - PnL calculations
  - liquidation threshold behavior.
- `INVARIANTS.md` update.

### Deliverables
- `/crates/instruments/`
- `/crates/positions/`
- portfolio snapshot API
- Docs with assumptions clearly stated.

### Rubric (25 pts)
- Accounting correctness — 10  
- Data model clarity — 7  
- Invariants & tests — 6  
- Documentation — 2  

---

## Week 3 — Exchange Adapter v0.1 (Orders, Acks, Fills) (35 pts)

### Learning Objectives
- Build a **direct exchange execution adapter**.
- Correctly handle exchange-specific quirks and failures.
- Separate “intent” from “exchange truth.”
- Emit execution state suitable for operator inspection.

### Background
- Crypto exchanges differ wildly in semantics.
- WebSocket ≠ truth; REST reconciliation is mandatory.
- Operators need visibility into rejects and fills in real time.

### Reading (Mon–Tue)
- OSTEP: retries, timeouts, partial failure.
- *Effective Rust*: error surfaces at system boundaries.

### Requirements (Wed–Thu)
- Implement an exchange adapter:
  - REST order submission (market/limit).
  - WebSocket order/fill updates.
  - Cancel support.
- Safety primitives:
  - rate limiting (orders/sec).
  - idempotent client order IDs.
- Typed error handling:
  - transport errors
  - exchange rejects
  - semantic mismatches.
- Strict mode:
  - if exchange connection uncertain → no new orders.
- Execution event stream:
  - expose order state transitions for UI (read-only).

### Testing & Docs (Fri)
- Mock exchange harness:
  - deterministic ack/fill/reject sequences.
  - reconnect mid-order.
- `RUNBOOK.md` updates:
  - “Exchange down”
  - “Reject spike”
  - “Rate limit hit”

### Deliverables
- `/crates/exchange_adapter/`
- execution event stream
- Mock exchange tests
- `DESIGN.md` update

### Rubric (35 pts)
- Order lifecycle correctness — 14  
- Safety primitives — 10  
- Robustness & tests — 7  
- Docs — 4  

---

## Week 4 — OMS + Reconciliation + Ledger (40 pts)

### Learning Objectives
- Maintain authoritative order & position state.
- Detect and react to **state divergence**.
- Provide an auditable trading ledger.
- Surface reconciliation status to operators.

### Background
- Crypto exchanges regularly desync.
- Reconciliation is not optional; it is survival.
- Operator awareness reduces blast radius.

### Reading (Mon–Tue)
- OSTEP: crash consistency.
- *Rust in Action*: persistence patterns (as needed).

### Requirements (Wed–Thu)
- OMS:
  - order state machine driven by exchange events.
  - restart-safe via WAL/snapshot.
- Reconciliation loop:
  - poll REST for open orders, balances, positions.
  - compare vs internal state.
  - divergence → halt trading.
- Ledger:
  - fills → positions → PnL.
  - fee accounting.
- Reconciliation status output:
  - healthy / degraded / halted.

### Testing & Docs (Fri)
- Integration tests:
  - missed WS fill recovered by REST.
  - restart + reconcile.
- `RUNBOOK.md`:
  - “Desync detected”
  - “Flatten positions”

### Deliverables
- `/crates/oms/`
- `/crates/ledger/`
- reconciliation worker
- reconciliation status feed

### Rubric (40 pts)
- OMS correctness — 14  
- Reconciliation behavior — 14  
- Ledger correctness — 8  
- Docs/tests — 4  

---

## Week 5 — Crypto Risk Rails (Liquidation, Funding, Kill Switch) (35 pts)

### Learning Objectives
- Enforce **hard risk limits** appropriate for crypto.
- Fail closed under uncertainty.
- Prevent liquidation events.
- Provide operator-visible risk state.

### Background
- Crypto losses are nonlinear and fast.
- Survival > Sharpe.
- Operators must see *why* trading is blocked.

### Reading (Mon–Tue)
- Chan: risk principles.
- *Rust Atomics & Locks*: concurrency review if needed.

### Requirements (Wed–Thu)
- Risk checks:
  - max leverage
  - liquidation buffer threshold
  - max position size
  - max order rate
  - max daily loss
- Funding exposure tracking (perps).
- Uncertainty halt:
  - data stale
  - exchange state unknown
  - reconciliation failing.
- Kill switch:
  - manual
  - automatic on hard breach.
- Risk state output:
  - reason codes for all denials.

### Testing & Docs (Fri)
- Boundary tests for each rule.
- Scenario tests:
  - liquidation buffer breach
  - stale feed halt.
- `RUNBOOK.md` update.

### Deliverables
- `/crates/risk_crypto/`
- risk state feed
- Updated metrics
- Updated runbook

### Rubric (35 pts)
- Risk correctness — 16  
- Fail-closed behavior — 10  
- Tests & determinism — 6  
- Docs — 3  

---

## Week 6 — Live “Armed Mode” Release + Operator Console (50 pts)

### Learning Objectives
- Run a live crypto system safely.
- Prove operability through incidents and replay.
- Provide an operator-grade Neo.mjs console.
- Produce a hiring-grade portfolio artifact.

### Background
- Live crypto trading is ops-heavy.
- Systems must be easy to stop and easy to audit.
- UI exists to support humans, not replace logic.

### Reading (Mon–Tue)
- *Effective Rust*: polish & documentation.
- OSTEP: reliability review.

### Requirements (Wed–Thu)
- Execution modes:
  - `--paper`
  - `--live`
  - `--armed` required to trade.
- Hard caps:
  - max leverage
  - max order size
  - max daily loss.
- Operator controls:
  - flatten all
  - safe shutdown.
- Deterministic recording:
  - market data
  - decisions
  - exchange events
  - operator commands.
- Neo.mjs operator console:
  - show mode (paper/live/armed)
  - show data health, reconciliation status, risk flags
  - display latency percentiles and reject rates
  - require confirmation for destructive actions.

### Testing & Docs (Fri)
- Live readiness checklist:
  - dry-run connects cleanly
  - no orders without `--armed`
  - replay reproduces session including operator actions.
- Two incident reports:
  - exchange desync
  - data stall.
- Final docs:
  - `README.md`
  - `DESIGN.md`
  - `RUNBOOK.md`
  - `EVAL.md`

### Deliverables
- Release tag `crypto-live-v0.1`
- Demo scripts
- Replay artifacts
- `/apps/terminal/` (Neo.mjs Operator Console)

### Rubric (50 pts)
- Safety & correctness — 18  
- End-to-end integration — 12  
- Operability — 10  
- Replay & evidence — 6  
- Documentation — 4  

## Crypto Live Execution v0.2 (6–8 Weeks) — Multi-Exchange Scaling + Neo.mjs Operator Terminal

> **Goal:** Extend Crypto Live Execution v0.1 into a **multi-exchange, latency-aware, capital-scaled trading system**
> operated safely by a small quant team:
> - Simultaneous trading across multiple crypto exchanges
> - Deterministic, explainable smart order routing
> - Inventory- and funding-aware execution
> - A **Neo.mjs operator terminal** for real-time observability, replay, and safe control
> - Strict **fail-closed** behavior under partial failures
>
> **Key Principle:**  
> v0.2 adds complexity **without rewriting architecture**.  
> All changes must compose onto v0.1.
>
> **Non-negotiables (entire stage)**
> - No `unwrap/expect` in library crates
> - Deterministic recording + replay of *all* data, decisions, exchange events, and operator actions
> - Continuous reconciliation vs each exchange’s REST truth
> - Explicit `--armed` mode for any live trading
> - UI is **observer + operator only**; never a source of truth
> - Loss of confidence (data, state, or exchange) ⇒ **halt**

---

## Week 1 — Multi-Exchange Market Data Fan-In + Health Modeling (30 pts)

### Learning Objectives
- Ingest market data from **multiple exchanges concurrently**.
- Normalize heterogeneous feeds into a unified internal schema.
- Detect **cross-venue staleness, gaps, and inconsistencies**.
- Surface venue health and confidence signals to the operator terminal.

### Background
- Multi-venue trading is the default in crypto.
- The challenge is not ingestion, but **trust**:
  - which venue is stale?
  - which quote is lagging?
  - when should trading stop entirely?

### Requirements (Wed–Thu)
- Extend market data ingestion to ≥2 exchanges:
  - independent WebSocket + REST adapters per venue
- Normalize all feeds into a canonical event stream.
- Maintain per-venue health metrics:
  - last update timestamp
  - message rate
  - disconnect / reconnect counters
- Global confidence model:
  - quorum-based “market healthy” signal
  - loss of quorum ⇒ halt trading
- Market health stream:
  - expose per-venue + global health snapshots for UI consumption.

### Testing & Docs (Fri)
- Simulate one venue stalling while others remain live.
- Verify deterministic halt behavior.
- Document quorum rules in `DESIGN.md`.

### Deliverables
- `/crates/md_fanin/`
- market health feed
- Updated `DESIGN.md`

### Rubric (30 pts)
- Fan-in correctness — 12  
- Failure detection & confidence modeling — 10  
- Tests & documentation — 8  

---

## Week 2 — Venue-Scoped OMS + Portfolio Aggregation (35 pts)

### Learning Objectives
- Track orders, fills, and positions **per exchange**.
- Prevent cross-venue state contamination.
- Present a clear global portfolio view to operators.

### Background
- Each exchange is its own failure domain.
- Global portfolio views must be derived, never assumed.

### Requirements (Wed–Thu)
- Extend OMS:
  - independent order state machines per venue
  - venue-scoped positions and balances
- Global portfolio aggregation:
  - net exposure
  - per-venue exposure
- Reconciliation:
  - independent REST reconciliation loops per exchange
  - divergence on any venue ⇒ partial or global halt (policy documented)
- Portfolio state stream:
  - per-venue + global summaries for UI.

### Testing & Docs (Fri)
- Missed fill on one venue recovered via reconciliation.
- Verify isolation guarantees.
- Update `INVARIANTS.md`.

### Deliverables
- Extended OMS + ledger
- portfolio aggregation feed
- Updated invariants

### Rubric (35 pts)
- OMS isolation correctness — 15  
- Reconciliation robustness — 12  
- Docs & tests — 8  

---

## Week 3 — Smart Order Routing (SOR) v0.1 + Operator Visibility (40 pts)

### Learning Objectives
- Route orders across venues using **price, liquidity, latency, and risk**.
- Keep routing decisions deterministic and explainable.
- Make routing behavior visible to operators.

### Background
- “Best price” is not always “best fill.”
- Operators must understand *why* an order went where it did.

### Requirements (Wed–Thu)
- Implement routing engine:
  - inputs: price, spread, venue health, inventory, latency stats
  - outputs: selected venue(s)
- Support:
  - single-venue routing
  - optional split routing
- Deterministic routing:
  - identical inputs ⇒ identical routing decision
- Routing metrics:
  - fill rate per venue
  - slippage per venue
  - latency per venue
- Routing explanation output:
  - structured reason codes exposed to UI.

### Testing & Docs (Fri)
- Scenario tests:
  - cheapest venue unhealthy
  - deepest venue slow
- Update `EVAL.md` with routing comparisons.

### Deliverables
- `/crates/router/`
- routing decision feed
- Routing evaluation report

### Rubric (40 pts)
- Routing correctness — 16  
- Determinism & explainability — 14  
- Docs & tests — 10  

---

## Week 4 — Latency Measurement, Modeling, and UI Visualization (35 pts)

### Learning Objectives
- Measure **end-to-end latency distributions**, not averages.
- Correlate latency with execution quality.
- Surface latency insights in the operator terminal.

### Background
- p99 latency kills strategies.
- Latency awareness is a routing and risk input, not a dashboard vanity metric.

### Requirements (Wed–Thu)
- Instrument timestamps:
  - data receive
  - decision
  - order send
  - ack
  - fill
- Produce latency histograms per venue.
- Add latency-aware routing constraint:
  - e.g., disallow venues with p95 > threshold
- Latency stream:
  - snapshot + rolling percentiles for UI display.

### Testing & Docs (Fri)
- Controlled latency injection tests.
- Before/after routing comparison using latency constraints.

### Deliverables
- Latency metrics in `/obs/`
- Latency feed
- `BENCH.md` latency section

### Rubric (35 pts)
- Measurement accuracy — 15  
- Latency-aware logic — 12  
- Docs & tests — 8  

---

## Week 5 — Inventory & Funding-Aware Execution + Operator Controls (35 pts)

### Learning Objectives
- Control inventory drift across venues.
- Minimize funding bleed in perpetual markets.
- Enable safe operator intervention.

### Background
- Many profitable strategies fail due to inventory mismanagement.
- Operators must see funding and inventory pressure building.

### Requirements (Wed–Thu)
- Inventory targets:
  - per symbol
  - per venue
- Funding model:
  - track accrued funding per venue
  - penalize routing into high-funding venues
- Inventory-aware routing:
  - bias toward underweight venues
- Emergency inventory flattening:
  - deterministic sequence
  - operator-triggerable via UI
- Inventory & funding state stream:
  - clear, read-only summaries for UI.

### Testing & Docs (Fri)
- Scenario tests:
  - funding spike
  - venue inventory imbalance
- `EVAL.md` funding impact analysis.

### Deliverables
- Inventory manager
- inventory/funding feed
- Updated risk + router integration

### Rubric (35 pts)
- Inventory correctness — 14  
- Funding modeling — 12  
- Docs & tests — 9  

---

## Week 6 — Capital Scaling, Risk Tiers, and Operator Governance (40 pts)

### Learning Objectives
- Scale capital safely without increasing systemic risk.
- Introduce explicit **risk tiers** and promotion gates.
- Make capital state and limits visible to operators.

### Background
- Scaling is a systems problem.
- Capital must move slower than confidence.

### Requirements (Wed–Thu)
- Risk tiers:
  - Tier 0 (paper)
  - Tier 1 (tiny live)
  - Tier 2 (scaled live)
- Per-tier limits:
  - max leverage
  - max order rate
  - max loss
- Capital allocator:
  - per-strategy caps
  - per-venue caps
- Promotion rules:
  - manual approval required
  - operator-visible state
- Capital state stream:
  - current tier
  - remaining risk budget.

### Testing & Docs (Fri)
- Tier promotion simulation.
- `RUNBOOK.md`: “Capital scale-up procedure.”

### Deliverables
- Capital allocator
- risk tier feed
- Updated runbook

### Rubric (40 pts)
- Scaling safety — 18  
- Risk correctness — 14  
- Docs & tests — 8  

---

## Week 7 — Failure Storms + Exchange Meltdown Scenarios + Replay (30 pts)

### Learning Objectives
- Prove survivability under extreme multi-venue failures.
- Practice professional incident response.
- Use replay + UI to explain what happened.

### Background
- Crypto markets regularly experience cascading failures.
- Systems must degrade predictably.

### Requirements (Wed–Thu)
- Failure storm scenarios:
  - one exchange offline
  - one exchange emitting bad data
  - mass rejects / throttling
- System behavior:
  - isolate failing venues
  - continue safely or halt globally
- Incident artifacts:
  - deterministic replay
  - operator timeline
- Neo.mjs replay inspector:
  - load session
  - scrub timeline
  - view health, routing, risk state over time.

### Deliverables
- Failure harness
- `/docs/incidents/`
- Replay artifacts + UI support

### Rubric (30 pts)
- Failure coverage — 12  
- Correct isolation — 10  
- Incident quality — 8  

---

## Week 8 — Crypto Production Release v0.2 + Operator Terminal v2 (60 pts)

### Learning Objectives
- Ship a **multi-venue, capital-scaled trading system** with a mature operator interface.
- Produce a hiring-grade production artifact.

### Requirements (Wed–Thu)
- Final system must support:
  - ≥2 exchanges
  - smart routing
  - latency-aware execution
  - inventory & funding controls
  - reconciliation everywhere
- Neo.mjs operator terminal v2:
  - global + per-venue health panels
  - routing explanation views
  - latency dashboards
  - inventory/funding summaries
  - risk tier status
  - operator commands (halt, flatten, resume) with confirmations
- End-to-end determinism:
  - replay reproduces system + UI state.

### Testing & Docs (Fri)
- Final documentation pack:
  - `README.md`
  - `DESIGN.md`
  - `RUNBOOK.md`
  - `INVARIANTS.md`
  - `EVAL.md`
  - `BENCH.md`
  - incident reports
- Demo scripts:
  - normal trading
  - failure storm
  - replay walkthrough.

### Deliverables
- Release tag `crypto-live-v0.2`
- Demo & replay artifacts
- `/apps/terminal/` (Neo.mjs Operator Terminal v2)

### Rubric (60 pts)
- End-to-end correctness — 20  
- Multi-venue robustness — 15  
- Performance & latency reasoning — 10  
- Risk & scaling discipline — 10  
- Documentation & evidence — 5  

