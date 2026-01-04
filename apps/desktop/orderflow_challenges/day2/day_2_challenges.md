

# Pre-Project Day — Determinism Plumbing: Seeded RNG, Stable Ordering, and “Golden” Replay Tests

## 1) Lecture Topic


* **Title (1 line):** Thread a seeded RNG through “agent intent → events” without leaking nondeterminism into the core

* **Why this matters for the project (2–3 bullets)**

  * Your simulator must prove: **same seed ⇒ same trace ⇒ same metrics**. That only works if randomness is **contained** and ordering is **stable**.
  * The easiest determinism bugs are accidental: unordered iteration, hidden time, or “randomness in the engine.”
  * A good replay test makes regressions obvious: you’ll catch “tiny refactors” that silently change behavior.

* **Required reading (must include exact sections)**

  * Reading 1: **[Effective Rust] — [Ch. 1 (Types) Items 1–3, 5, 9]** → “Takeaway rule for today”: *Make nondeterminism impossible by design—push it to edges and encode contracts in types (`Result`, explicit inputs/outputs).*
  * Reading 2: **[Rust for Rustaceans] — [Ch. 6 (Testing)]** → “Takeaway rule for today”: *Prefer tests that lock down behavior (golden/snapshot-style) when refactors could change outputs.*
  * Reading 3: **[Rust in Action] — [Ch. 2 (Language foundations) + Ch. 3 (Compound data types)]** → “Takeaway rule for today”: *Use explicit data structures and `match`/loops to keep control flow understandable and reproducible.*

* **Key concepts (5–8 bullets)**

  * **RNG boundary rule:** RNG lives in “policy” (agent intent), not in “mechanics” (engine apply)
  * **Seed threading:** pass `&mut Rng` (or your wrapper) explicitly—no globals
  * **Deterministic schedules:** if two agents act in a tick, define a stable order (e.g., sorted by `AgentId`)
  * **Stable collections:** avoid relying on `HashMap` iteration for traces; use `Vec` + sort or `BTreeMap`
  * **Golden replay test:** fixed seed + fixed steps ⇒ exact expected fingerprint/trace
  * **Pure core API:** `apply(state, event)` is deterministic; `agent_step(state, rng)` is the only place randomness enters
  * **Error strategy:** typed errors or explicit `Reject/NoOp` events—never “silent ignore”
  * **Clippy mindset:** write small, explicit functions; avoid “clever” nondeterministic shortcuts

* **Tiny demo (optional, ≤10 lines)**

```rust
fn pick_id(rng: &mut u64) -> u32 {
    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*rng >> 32) as u32
}
```

* **Discussion prompt of the day (from project outline)**

  * **Prompt:** *Determinism: why `HashMap` iteration can break reproducibility, and when to prefer `BTreeMap` or sorting?*
  * **What a strong answer includes (2–3 bullets)**

    * `HashMap` iteration order is not a stable contract; it can change across runs/builds/platforms.
    * For trace output, either **sort keys** or use an **ordered structure** (`BTreeMap`) so serialization is stable.
    * Names the failure mode: “same seed, different printed trace” because output order drifted.

* **“Prove you learned it” checklist (3 bullets)**

  * I can implement a tiny seeded RNG and show two runs with the same seed generate the same first N numbers.
  * I can create a golden test that asserts an exact fingerprint string for a fixed seed + fixed steps.
  * **Reading-based proof:** I can cite **Rust for Rustaceans — Ch. 6 (Testing)** and explain why a golden/snapshot test is appropriate for determinism regressions.

---

## 2) Quiz (5 Questions)

* **Q1) You have `apply(state, event)` and `agent_step(state, rng)`. Where is randomness allowed, and why is it forbidden in the other function?**

  * **Tags:** Reasoning + Difficulty 3 + Topics: determinism, architecture boundaries
  * **Answer key:**

    * Randomness allowed in `agent_step` to generate *intent/events* under a seed.
    * Forbidden in `apply` because it must be a deterministic state machine given inputs.
    * Mixing randomness into `apply` breaks replay and makes tests flaky/unreproducible.

* **Q2) (Reading) What is a “golden” test in practice, and why is it especially useful for deterministic simulators?**

  * **Tags:** Reading + Difficulty 2 + Topics: testing, determinism, regression
  * **Answer key:**

    * A test that asserts an exact expected output (string/trace/fingerprint) for fixed inputs.
    * Useful because refactors can subtly change behavior even when types still compile.
    * For determinism, it catches “behavior drift” immediately and makes it easy to bisect.
  * **Reading anchor:** Rust for Rustaceans — Ch. 6 (Testing); applies because it emphasizes high-signal tests that lock down behavior across refactors.

* **Q3) Suppose you collect per-symbol metrics in a `HashMap` and then serialize them. You notice trace diffs across runs with the same seed. What are two deterministic fixes?**

  * **Tags:** Bug-Spotting + Difficulty 3 + Topics: ordering, collections, serialization
  * **Answer key:**

    * Convert to `Vec<(K,V)>` and sort by key before printing/serializing.
    * Use `BTreeMap` for ordered iteration.
    * Ensure any derived ordering is explicitly defined (not “whatever the map gives”).

* **Q4) (Discussion) Your replay file must be byte-for-byte identical for the same seed, but you also want to print “events/sec.” How do you design this so replay stays stable, and what tradeoff are you making?**

  * **Tags:** Discussion + Difficulty 4 + Topics: observability, determinism, metrics
  * **Answer key:**

    * Keep replay/trace purely deterministic (tick index + events); exclude wall-clock from trace.
    * If you report events/sec, compute it outside trace (display-only) or report deterministic “events/tick.”
    * Tradeoff: wall-clock numbers become non-replayable and shouldn’t be used to validate correctness.

* **Q5) (Reading) Effective Rust pushes type-driven design. What type/signature choice best enforces “no hidden nondeterminism,” and why?**

  * **Tags:** Reading + Difficulty 3 + Topics: API design, explicit inputs, `Result`
  * **Answer key:**

    * Make nondeterministic dependencies explicit parameters (e.g., `fn step(..., rng: &mut Rng) -> Result<...>`).
    * Return `Result` for invariant violations instead of panicking.
    * Explicit inputs/outputs make behavior testable and replayable.
  * **Reading anchor:** Effective Rust — Ch. 1 (Types) Items 1–3, 5, 9; applies because it stresses encoding contracts in types and making important dependencies explicit.

---

## 3) Mini-Challenges (2)

### Mini-Challenge 1 — Tiny Seeded RNG + Repeatability Test

* **Goal**

  * Implement a minimal deterministic RNG (LCG-style) and prove repeatability with tests.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day2_seeded_rng && cd day2_seeded_rng`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Create `struct Rng { state: u64 }` with:

    * `fn new(seed: u64) -> Self`
    * `fn next_u32(&mut self) -> u32` using a simple LCG update:

      * `state = state * A + C` using `wrapping_mul`/`wrapping_add`
      * return upper 32 bits as `u32`
  * Tests:

    * Same seed produces the same first 5 outputs (assert exact `Vec<u32>`)
    * Different seeds produce a different first output (assert `!=`)
* **Proof (what to run / what output must show)**

  * `cargo test` passes
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic: no wall-clock, no randomness beyond your `Rng`
  * Keep code small and readable (clippy-friendly)
* **What skill it builds for the project (1 line)**

  * Seed plumbing you’ll later use for `--seed` determinism end-to-end.

---

### Mini-Challenge 2 — Golden Replay Fingerprint with Stable Ordering (Reading Proof)

* **Goal**

  * Build a tiny “agent generates events using RNG → engine applies” pipeline and lock it down with a golden test.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day2_golden_replay && cd day2_golden_replay`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Reuse your `Rng` from Challenge 1 (copy/paste OK).
  * Define:

    * `enum Event { New(u32), Cancel(u32) }`
    * `struct State { open: Vec<u32> }`
    * `fn apply(state: &mut State, e: Event) -> Result<(), &'static str>` (no panics)
  * Implement `fn agent_step(rng: &mut Rng, tick: u32) -> Event`:

    * If `tick % 3 == 0` → `New(next_u32 % 10)`
    * Else → `Cancel(next_u32 % 10)`
  * Implement `fn run(seed: u64, ticks: u32) -> Result<String, &'static str>`:

    * Runs ticks, collects events in a `Vec<Event>` (in order), applies them
    * Returns a fingerprint string that is **stable**:

      * include `events=<count>;open_sorted=[...]`
      * IMPORTANT: `open_sorted` must be sorted before formatting
  * Add a golden test for `seed=42, ticks=12` asserting the exact fingerprint string.
  * Add a short comment citing the reading: one sentence like

    * `// RfR Ch.6: golden tests lock down behavior across refactors; we assert an exact fingerprint.`
* **Proof (what to run / what output must show)**

  * `cargo test` passes
  * Golden assertion compares to an exact string (byte-for-byte)
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic output (sorting before formatting)
  * No `HashMap` iteration for anything you serialize
* **Reading link (required for at least 1 challenge)**

  * **Anchor:** Rust for Rustaceans — Ch. 6 (Testing)
  * **How it changes your implementation (1 line):** You must write a golden test that asserts an exact, stable output so determinism regressions are caught immediately.
* **What skill it builds for the project (1 line)**

  * End-to-end determinism proof pattern: seed → events → apply → stable serialized summary.
