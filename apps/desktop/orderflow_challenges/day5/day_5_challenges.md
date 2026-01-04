
# Pre-Project Day — Production Readiness: Typed Errors, No-Panic Core, and Actionable Runbooks

## 1) Lecture Topic

* **Title (1 line):** Make the core “uncrashable”: typed errors, no `unwrap`, deterministic outputs, and an ops-friendly failure story

* **Why this matters for the project (2–3 bullets)**

  * Your rubric explicitly rewards robustness and polish; “works on my machine” isn’t enough when determinism is a requirement.
  * The fastest way to lose trust is a simulator that panics or produces non-reproducible traces on edge cases.
  * A small RUNBOOK now saves hours later: every bug should be reproducible by seed + config.

* **Required reading (must include exact sections)**

  * Reading 1: **[Effective Rust] — [Ch. 1 (Types) Items 1–3, 5, 9]** → “Takeaway rule for today”: *Prefer explicit error types and `Result` plumbing over panics; make contracts visible in signatures.*
  * Reading 2: **[Rust for Rustaceans] — [Ch. 6 (Testing)]** → “Takeaway rule for today”: *Use high-signal tests (golden/behavior locks) so refactors don’t quietly change outcomes.*
  * Reading 3: **[Rust in Action] — [Ch. 2 (Language foundations)]** → “Takeaway rule for today”: *Control flow should be explicit and readable—especially on errors and edge cases.*

* **Key concepts (5–8 bullets)**

  * **Library vs shell:** core returns `Result<Summary, SimErr>`; CLI decides exit codes + printing
  * **Typed errors:** `enum SimErr { UnknownOrderId, InvalidConfig, InvariantViolated, ... }`
  * **No hidden panics:** eliminate `unwrap/expect` in core; replace with `?` and explicit mapping
  * **Deterministic failure behavior:** on invalid event/action, choose deterministic policy (`Reject`, `NoOp`, or `Err`) and document it
  * **Stable formatting:** sort before printing; don’t include nondeterministic data in the trace
  * **“Repro recipe”:** every bug report must include seed, ticks, and config parameters
  * **Clippy/rustfmt gates:** treat warnings as build failures; keep APIs small and clear
  * **Minimal runbook:** common failure modes + how to validate determinism

* **Tiny demo (optional, ≤10 lines)**

```rust
fn parse_u32(s: &str) -> Result<u32, SimErr> {
    s.parse::<u32>().map_err(|_| SimErr::BadNumber(s.to_string()))
}
```

* **Discussion prompt of the day (from project outline)**

  * **Prompt:** *Error strategy: `Reject` event vs `Result::Err`—when each is better (replayability, metrics, UX)?*
  * **What a strong answer includes (2–3 bullets)**

    * Picks a primary strategy and states a rule (e.g., “invalid actions become `Reject` events; only programmer bugs are `Err`”).
    * Connects to replay: `Reject` keeps failures in trace; `Err` may stop the run unless also recorded.
    * Mentions failure mode: silent ignores create “clean looking” metrics that are wrong.

* **“Prove you learned it” checklist (3 bullets)**

  * I can refactor parsing/validation to use `Result` + typed errors (no panics) and test error variants.
  * I can write a golden test that locks down deterministic output/fingerprint and survives refactors.
  * **Reading-based proof:** I can cite **Effective Rust — Ch. 1 Items 1–3, 5, 9** and show one place where `?` + typed error improved the design.

---

## 2) Quiz (5 Questions)

* **Q1) In “library-style core” code, why is `unwrap()` worse than returning `Result` even if you “know it can’t fail”? Give two production failure modes.**

  * **Tags:** Tradeoff + Difficulty 3 + Topics: robustness, errors, ops
  * **Answer key:**

    * “Can’t fail” assumptions break under new inputs, refactors, or corrupted state.
    * Panics destroy reproducibility (no clean trace) and are hard to triage remotely.
    * Failure modes: bad config, unexpected empty collections, parse errors, edge-case ordering bugs.

* **Q2) (Reading) Effective Rust emphasizes type-driven contracts. What does a good error type buy you over `anyhow::Error` or string errors in a small simulator core?**

  * **Tags:** Reading + Difficulty 3 + Topics: typed errors, API design
  * **Answer key:**

    * Callers can match on error variants deterministically and decide policy.
    * Tests can assert exact failure modes (stable, high-signal).
    * RUNBOOK becomes actionable (“if SimErr::UnknownOrderId then check cancel logic”).
  * **Reading anchor:** Effective Rust — Ch. 1 (Types) Items 1–3, 5, 9; applies because it argues for meaning-carrying types and explicit `Result` plumbing.

* **Q3) You’re asked to add “events/sec.” Why must this be treated as display-only (or computed as deterministic events/tick) if you want byte-for-byte replay files?**

  * **Tags:** Reasoning + Difficulty 4 + Topics: determinism, metrics, observability
  * **Answer key:**

    * Wall-clock depends on machine load, printing, OS scheduling—non-reproducible.
    * Including it in trace changes bytes across runs, breaking replay diffs.
    * Deterministic alternative: compute events/tick or keep wall-clock in stderr display only.

* **Q4) (Discussion) Choose one policy: invalid cancels become `Reject` events OR become `Err`. Defend your choice for replay + metrics + UX, and name one downside.**

  * **Tags:** Discussion + Difficulty 4 + Topics: error strategy, replayability, product behavior
  * **Answer key:**

    * If `Reject`: replay stays complete, metrics can count rejects, UX can show “why rejected.”
    * Downside: event stream grows and you must define semantics for rejects.
    * If `Err`: simpler core; downside is runs can stop and you lose “complete trace” unless you also record error events.

* **Q5) (Reading) From Rust for Rustaceans testing mindset: what makes a golden test “high signal,” and what’s one way to avoid brittle golden tests?**

  * **Tags:** Reading + Difficulty 3 + Topics: testing, determinism, snapshot strategy
  * **Answer key:**

    * High signal: asserts behavior that matters (ordering, counts, fingerprint) with fixed inputs/seed.
    * Avoid brittleness by snapshotting a stable fingerprint (sorted/normalized) rather than verbose raw dumps.
    * Keep golden scope small: first N events + summary, not megabytes.
  * **Reading anchor:** Rust for Rustaceans — Ch. 6 (Testing); applies because it focuses on behavior-locking tests that survive refactors while catching regressions.

---

## 3) Mini-Challenges (2)

### Mini-Challenge 1 — Typed Parse + Validation Pipeline (No Panics)

* **Goal**

  * Practice “core returns typed errors; shell decides” by building a tiny parser/validator that never panics.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day5_typed_errors && cd day5_typed_errors`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Create:

    * `enum SimErr { EmptyInput, BadNumber(String), OutOfRange { val: u32, min: u32, max: u32 } }`
  * Implement:

    * `fn parse_rate(s: &str, min: u32, max: u32) -> Result<u32, SimErr>`

      * trims input; empty → `EmptyInput`
      * parse u32; failure → `BadNumber(original)`
      * if outside range → `OutOfRange{...}`
  * Tests must assert exact error variants and fields for:

    * `""`, `"abc"`, `"999"` with max 100
    * success case `"42"`
* **Proof (what to run / what output must show)**

  * `cargo test` passes
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Use `?` + `map_err` rather than manual branching everywhere
  * Deterministic: no randomness, no time, no printing
* **What skill it builds for the project (1 line)**

  * Clean config/CLI validation that fails predictably and is easy to debug.

---

### Mini-Challenge 2 — Mini Runbook + Golden Determinism Gate (Reading Proof)

* **Goal**

  * Create a reproducibility “contract” artifact: seed + inputs ⇒ stable fingerprint, plus a tiny runbook note showing how to debug.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day5_runbook_golden && cd day5_runbook_golden`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
  * Create a `RUNBOOK.md` in the scratch crate root (yes, in this scratch folder only)
* **Requirements (clear, testable)**

  * Implement:

    * a tiny seeded `Rng` (copy from Day 2)
    * `fn run(seed: u64, ticks: u32) -> String` that:

      * generates a deterministic sequence of 10–30 simple events (you can reuse `tick % k` rules)
      * returns a stable **fingerprint** string that is normalized (e.g., sorted list of open IDs, counts)
  * Add a golden test asserting `run(123, 20)` equals an exact string.
  * Write `RUNBOOK.md` with exactly **3 sections**:

    1. **Repro steps**: include the seed + ticks used in the golden test
    2. **If the golden test fails**: 3 bullet checklist (what changed? ordering? normalization?)
    3. **Policy note**: one sentence stating whether invalid actions become `Err` or `Reject` (pick one)
  * Add a code comment near the golden test:

    * `// RfR Ch.6: golden tests lock down behavior; normalize before formatting to avoid brittle snapshots.`
* **Proof (what to run / what output must show)**

  * `cargo test` passes
  * `RUNBOOK.md` exists and includes the exact seed/ticks from your golden test
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic formatting: sort/normalize before building the fingerprint string
  * No printing/logging in tests
* **Reading link (required for at least 1 challenge)**

  * **Anchor:** Rust for Rustaceans — Ch. 6 (Testing)
  * **How it changes your implementation (1 line):** You must normalize outputs and lock them with a golden test so behavior changes are deliberate and reviewable.
* **What skill it builds for the project (1 line)**

  * “Ops mindset”: reproducible failures, stable outputs, and documentation that turns bugs into steps—not mysteries.
