
# Pre-Project Day — Metrics + Invariants: Counting Correctly Without Breaking Determinism

## 1) Lecture Topic

* **Title (1 line):** Build metrics as a pure “event reducer” and enforce invariants with tests (no I/O, no panics)

* **Why this matters for the project (2–3 bullets)**

  * Your rubric rewards “metrics + regime progression” and **robustness**—bad counters make the whole simulator untrustworthy.
  * Metrics are where subtle bugs hide: off-by-one, double-counting, and “derived rates” that silently go out of bounds.
  * A pure reducer (`Metrics::apply(event)`) is deterministic, testable, and lets you prove invariants early.

* **Required reading (must include exact sections)**

  * Reading 1: **[Rust for Rustaceans] — [Ch. 1 (Foundations)]** → “Takeaway rule for today”: *Write invariants down and test them; don’t depend on “it probably won’t happen.”*
  * Reading 2: **[Effective Rust] — [Ch. 1 (Types) Items 1–3, 5, 9]** → “Takeaway rule for today”: *Use types + `Result` to represent invalid transitions; avoid panics and make failures explicit.*
  * Reading 3: **[Rust in Action] — [Ch. 3 (Compound data types)]** → “Takeaway rule for today”: *Model state as structs/enums so updates are explicit and mechanically checkable.*

* **Key concepts (5–8 bullets)**

  * **Metrics as reducer:** update counters from each event; avoid “recount by scanning state” in hot path
  * **Derived metrics:** cancel rate, queue depth, throughput—computed from counters + known time basis
  * **Deterministic throughput:** prefer “events/tick” inside trace; wall-clock is display-only
  * **Invariant examples:** cancel rate ∈ [0,1], fills ≤ new orders (or justify partial fills), depth never negative
  * **Separation of concerns:** engine emits events; metrics consumes events; output formats summary
  * **Typed errors vs reject events:** choose one, but keep it deterministic and testable
  * **Clippy-friendly design:** small functions; explicit conversions; no hidden mutation
  * **Testing strategy:** table-driven unit tests + one integration script test

* **Tiny demo (optional, ≤10 lines)**

```rust
#[derive(Default)]
struct Metrics { new: u64, cancel: u64, fill: u64, depth: i64 }
fn apply(m: &mut Metrics, e: Event) {
    match e { Event::New => {m.new+=1; m.depth+=1;}
              Event::Cancel => {m.cancel+=1; m.depth-=1;}
              Event::Fill => {m.fill+=1; m.depth-=1;} }
}
```

* **Discussion prompt of the day (from project outline)**

  * **Prompt:** *Invariants: pick 3 invariants for cancels/fills and justify how you’ll enforce them (error vs reject/no-op).*
  * **What a strong answer includes (2–3 bullets)**

    * Chooses 3 concrete invariants (e.g., “depth never negative”, “cancel references open order”, “fills only for open orders”).
    * States enforcement mechanism: either `Result<_, DomainErr>` or deterministic `Reject/NoOp` events—no silent ignore.
    * Mentions failure mode: “double-decrement depth” or “cancel unknown order” causing negative depth and bogus rates.

* **“Prove you learned it” checklist (3 bullets)**

  * I can implement `Metrics::apply(Event)` and prove counters match a known event script.
  * I can write tests that fail on invariant violations (negative depth, rates out of range).
  * **Reading-based proof:** I can cite **Rust for Rustaceans — Ch. 1 (Foundations)** and list at least 5 invariants I would include in DESIGN.md.

---

## 2) Quiz (5 Questions)

* **Q1) You’re computing cancel rate as `cancels / new_orders`. What are two edge cases that can produce wrong answers, and how do you handle them deterministically?**

  * **Tags:** Bug-Spotting + Difficulty 3 + Topics: metrics, numerics, invariants
  * **Answer key:**

    * `new_orders == 0` → division by zero (or meaningless rate).
    * Integer division truncation if using integers.
    * Fix: define policy (e.g., rate = 0.0 when new_orders == 0), use `f64` with explicit cast, and test it.

* **Q2) (Reading) Why does “no hidden panics” matter more for metrics than almost anywhere else in the simulator?**

  * **Tags:** Reading + Difficulty 3 + Topics: robustness, `Result`, invariants
  * **Answer key:**

    * Metrics code runs constantly; a single panic kills runs and breaks replay workflows.
    * Panics hide which invariant failed and make debugging non-actionable.
    * Typed errors or deterministic reject/no-op events preserve traceability and reproducibility.
  * **Reading anchor:** Effective Rust — Ch. 1 (Types) Items 1–3, 5, 9; applies because it emphasizes explicit error handling and type-driven design instead of panics.

* **Q3) Your queue depth metric goes negative in a test. List two plausible root causes in an event-driven simulator.**

  * **Tags:** Reasoning + Difficulty 3 + Topics: invariants, state transitions
  * **Answer key:**

    * Applying `Cancel` or `Fill` for an order that wasn’t open (unknown id or already removed).
    * Double-applying an event (e.g., replay bug) or decrementing depth in two places.
    * Incorrect order lifecycle: fill without new, cancel after fill not rejected.

* **Q4) (Discussion) You want to model “Reject” deterministically. Should rejections be returned as `Err` or emitted as `Event::Reject`? Choose one and defend it (replay, metrics, UX).**

  * **Tags:** Discussion + Difficulty 4 + Topics: tradeoffs, event modeling, replay
  * **Answer key:**

    * `Event::Reject` keeps everything in the trace; metrics can count rejects; replay stays simple.
    * `Err` is simpler for core correctness but can complicate replay unless you convert to a trace record.
    * Strong answer explicitly states a rule and keeps it consistent (no silent ignore).

* **Q5) (Reading) From Rust for Rustaceans’ “invariants first” mindset: what’s the difference between an invariant and a metric, and why must invariants be test-enforced?**

  * **Tags:** Reading + Difficulty 2 + Topics: invariants, testing discipline
  * **Answer key:**

    * Invariant: must always hold (safety/correctness). Metric: observational measurement that can vary.
    * Invariants prevent invalid state; metrics describe state.
    * Tests enforce invariants because otherwise failures show up as “weird numbers” far downstream.
  * **Reading anchor:** Rust for Rustaceans — Ch. 1 (Foundations); applies because it frames professional habits around explicit invariants and tests as guardrails.

---

## 3) Mini-Challenges (2)

### Mini-Challenge 1 — Metrics Reducer + Table-Driven Tests

* **Goal**

  * Implement a pure metrics reducer and prove it matches expected counts for scripted event sequences.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day4_metrics_reducer && cd day4_metrics_reducer`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Define:

    * `enum Event { New, Cancel, Fill }`
    * `#[derive(Default)] struct Metrics { new: u64, cancel: u64, fill: u64, depth: i64 }`
  * Implement `fn apply(m: &mut Metrics, e: Event)`

    * Update counters + depth changes deterministically
  * Add table-driven tests:

    * Script A: `[New, New, Cancel]` → `new=2,cancel=1,fill=0,depth=1`
    * Script B: `[New, Fill]` → `new=1,fill=1,depth=0`
* **Proof (what to run / what output must show)**

  * `cargo test` passes
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * No printing/logging in tests
  * Keep reducer pure and tiny (single responsibility)
* **What skill it builds for the project (1 line)**

  * Correct counter updates and “metrics as reducer” architecture you’ll reuse directly.

---

### Mini-Challenge 2 — Invariant Gate: Detect Negative Depth + Reading Proof

* **Goal**

  * Add a deterministic invariant check layer that prevents invalid metric states and produces actionable failures.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day4_invariant_gate && cd day4_invariant_gate`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Define:

    * `enum Event { New, Cancel, Fill }`
    * `#[derive(Default)] struct Metrics { depth: i64, new: u64, cancel: u64, fill: u64 }`
    * `enum InvErr { NegativeDepth { after: i64, event: Event } }`
  * Implement `fn apply_checked(m: &mut Metrics, e: Event) -> Result<(), InvErr>`:

    * Apply the same updates as Day 4 Challenge 1
    * If `m.depth < 0` after applying, return `Err(InvErr::NegativeDepth { ... })`
  * Tests:

    * Script `[Cancel]` must return `Err(NegativeDepth { after: -1, event: Cancel })`
    * Script `[New, Cancel]` must return `Ok(())` and end at depth 0
  * Add a short comment above `InvErr`:

    * `// RfR Ch.1: invariants are explicit and test-enforced; we fail fast with typed errors.`
* **Proof (what to run / what output must show)**

  * `cargo test` passes
  * The failing script asserts the exact error variant and fields
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic: no randomness, no time
  * Typed errors only (no stringly-typed “oops”)
* **Reading link (required for at least 1 challenge)**

  * **Anchor:** Rust for Rustaceans — Ch. 1 (Foundations)
  * **How it changes your implementation (1 line):** You must encode invariants as explicit checks that fail deterministically with typed errors and tests, rather than relying on debugging output.
* **What skill it builds for the project (1 line)**

  * Enforcing correctness constraints early—so your simulator never produces “plausible but wrong” metrics.
