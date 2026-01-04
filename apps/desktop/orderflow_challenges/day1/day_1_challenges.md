
# Pre-Project Day — Deterministic Modeling: Events, State, and `match`

## 1) Lecture Topic

* **Title (1 line):** Model an event-driven core with enums + `match`, and make determinism a first-class constraint

* **Why this matters for the project (2–3 bullets)**

  * Your simulator lives or dies on **clear event/state modeling**—if types are fuzzy, invariants and tests become impossible.
  * Determinism isn’t a “feature later”: small choices (iteration order, IDs, branching) can silently break reproducible traces.
  * If you can express transitions as **pure functions**, you can test mechanics and metrics without any I/O.

* **Required reading (must include exact sections)**

  * Reading 1: **[Rust in Action] — [Ch. 2 (Language foundations) + Ch. 3 (Compound data types)]** → “Takeaway rule for today”: *Use enums + `match` to encode event variants explicitly; avoid ad-hoc strings that hide invalid states.*
  * Reading 2: **[Effective Rust] — [Ch. 1 (Types) Items 1–3, 5, 9]** → “Takeaway rule for today”: *Let types carry meaning; prefer `Option/Result` and iterator transforms to make invalid transitions unrepresentable or explicit.*
  * Reading 3: **[Rust for Rustaceans] — [Ch. 1 (Foundations)]** → “Takeaway rule for today”: *Write down invariants up front; design the code so panics aren’t “control flow.”*

* **Key concepts (5–8 bullets)**

  * **Event enum** as your contract: `Event::NewOrder`, `Event::Cancel`, `Event::Fill` (and later optional variants)
  * **State struct** as your single source of truth (no “hidden globals”)
  * **Transition function** shape: `fn apply(state: &mut State, e: Event) -> Result<(), DomainErr>`
  * **Deterministic IDs**: sequential counters or seeded generation—never “whatever happened to come first”
  * **Deterministic ordering**: if you need ordering, choose `Vec`/sorting, or `BTreeMap`—don’t rely on map iteration
  * **Invariants**: properties that must always hold (e.g., cancels only cancel open orders)
  * **Errors vs events**: when to model failures as `Reject` events vs returning `Err`
  * **No unwrap/expect in core**: core returns typed errors; shell decides how to print/exit

* **Tiny demo (optional, ≤10 lines)**

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event { New(u32), Cancel(u32) }

fn apply(open: &mut Vec<u32>, e: Event) -> Result<(), &'static str> {
    match e {
        Event::New(id) => { open.push(id); Ok(()) }
        Event::Cancel(id) => open.iter().position(|&x| x == id)
            .map(|i| { open.remove(i); })
            .ok_or("unknown id"),
    }
}
```

* **Discussion prompt of the day (from project outline)**

  * **Prompt:** *Seed plumbing: where randomness is allowed vs forbidden (scenario: what belongs in Agent vs Engine vs Output?)*
  * **What a strong answer includes (2–3 bullets)**

    * Randomness is **allowed only at controlled boundaries** (e.g., agent intent generation) and always via a seeded RNG passed in.
    * Engine/state transitions must be **pure given (state, event)**; output formatting must not affect state or ordering.
    * Identifies failure mode: “accidental randomness” via unordered iteration, timestamps, or sampling wall-clock.

* **“Prove you learned it” checklist (3 bullets)**

  * I can write an `Event` enum + `match`-based `apply()` that returns `Result` (no panics) and has at least **one invariant test**.
  * I can explain (in 2–3 sentences) why unordered iteration can break reproducible traces and how I avoid it.
  * **Reading-based proof:** I can cite **Effective Rust — Ch. 1 Items 1–3, 5, 9** and point to one place where a stronger type (`enum`, `Option`, `Result`) prevented an invalid state.

---

## 2) Quiz (5 Questions)

* **Q1) You’re tempted to represent events as `("NEW", id)` and `("CANCEL", id)` strings. What breaks first when the system grows, and what’s the type-driven fix?**

  * **Tags:** Concept + Difficulty 2 + Topics: enums, `match`, modeling
  * **Answer key:**

    * Strings allow invalid variants at runtime (typos, missing fields, partial parsing).
    * You lose compiler exhaustiveness checking; adding a new event becomes “silent partial support.”
    * Fix: `enum Event { NewOrder{...}, Cancel{...}, Fill{...} }` and central `match` handling.

* **Q2) (Reading) In one sentence: what’s the practical advantage of `match` exhaustiveness for an event loop? Then give one concrete bug it prevents.**

  * **Tags:** Reading + Difficulty 2 + Topics: `match`, maintainability, correctness
  * **Answer key:**

    * Exhaustiveness forces you to handle every event variant explicitly when you evolve the protocol.
    * Prevents “new variant compiles but is ignored,” causing missing metrics updates or inconsistent state.
    * Encourages a single transition point (`apply`) rather than scattered conditionals.
  * **Reading anchor (ONLY for Reading-tagged questions):** Rust in Action — Ch. 2 + Ch. 3; applies because it teaches enum/`match` modeling patterns that make state machines explicit.

* **Q3) Suppose `Cancel(id)` arrives for an order that was never opened (or already filled). In a deterministic simulator core, what are two acceptable behaviors and what are the tradeoffs?**

  * **Tags:** Tradeoff + Difficulty 3 + Topics: invariants, error handling, determinism
  * **Answer key:**

    * Behavior A: return `Err(DomainErr::UnknownOrder)`; shell can log and continue/stop deterministically.
    * Behavior B: convert into a deterministic `Reject/NoOp` event recorded in trace.
    * Tradeoff: `Err` simplifies core but may complicate replay; `Reject` keeps everything in the event stream for post-mortems.

* **Q4) (Discussion) You need events/sec throughput. Measuring wall-clock time can break determinism. How would you design throughput reporting so runs stay reproducible, and what failure mode are you avoiding?**

  * **Tags:** Discussion + Difficulty 4 + Topics: observability, determinism, metrics
  * **Answer key:**

    * Keep **simulation time** tick-based and deterministic; compute “events per tick” as deterministic metric.
    * If wall-clock is reported, treat it as **non-authoritative** and keep it out of trace/logic (display-only).
    * Failure mode: trace differs because timing/printing/logging changes event scheduling or ordering.

* **Q5) (Reading) Effective Rust pushes type-driven design. In this context, when is `Option` better than `Result` inside `apply()`, and when is `Result` better? Give one example each.**

  * **Tags:** Reading + Difficulty 3 + Topics: `Option`, `Result`, API design
  * **Answer key:**

    * `Option` when “absence is normal” and not an error (e.g., `find_open_order(id) -> Option<OrderRef>`).
    * `Result` when you need a typed failure reason to enforce invariants (e.g., `apply_cancel(id) -> Result<(), DomainErr>`).
    * Using `Result` improves debugging and RUNBOOK quality (“why did this reject happen?”).
  * **Reading anchor (ONLY for Reading-tagged questions):** Effective Rust — Ch. 1 Items 1–3, 5, 9; applies because it emphasizes choosing types that encode meaning and using `Option/Result` transforms idiomatically.

---

## 3) Mini-Challenges (2)

### Mini-Challenge 1 — Enum State Machine: `apply_event` with Typed Errors

* **Goal**

  * Build the smallest “event → state transition” core you can test, with **no panics** and deterministic behavior.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day1_apply_event && cd day1_apply_event`
  * Implement logic in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Define:

    * `enum Event { New(u32), Cancel(u32) }`
    * `struct State { open: Vec<u32> }`
    * `enum DomainErr { UnknownId(u32) }`
  * Implement `fn apply(state: &mut State, e: Event) -> Result<(), DomainErr>`

    * `New(id)` adds to `open`
    * `Cancel(id)` removes if present, else returns `Err(DomainErr::UnknownId(id))`
  * Add **2 tests**:

    * canceling an existing id removes it
    * canceling an unknown id returns the correct error and does not change state
* **Proof (what to run / what output must show)**

  * `cargo test` passes
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic output: tests must not depend on printing/logging
  * Keep functions small and single-purpose
* **What skill it builds for the project (1 line)**

  * Pure, testable transition logic—the heart of the simulator engine.

---

### Mini-Challenge 2 — Deterministic Trace Fingerprint (with Reading Proof)

* **Goal**

  * Prove you can produce a **stable, reproducible trace summary** from a deterministic transition function.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day1_trace_fingerprint && cd day1_trace_fingerprint`
  * Logic in `src/lib.rs`, tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Reuse `Event`, `State`, `DomainErr` from Challenge 1 (copy/paste is fine).
  * Implement `fn run_script(events: &[Event]) -> Result<String, DomainErr>` that:

    * Starts from `State { open: vec![] }`
    * Applies all events in order
    * Returns a **fingerprint string** like: `"open=[2,5];len=2"`
  * Add **1 golden test**:

    * Given a fixed script (you choose 6–10 events), the returned fingerprint matches an exact expected string.
  * Add a short comment at the top of `run_script` explaining why this function is “pure core” (1–2 sentences).
* **Proof (what to run / what output must show)**

  * `cargo test` passes
  * The golden test asserts an exact, stable fingerprint string
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Don’t use `HashMap` here (stick to `Vec` so ordering is obvious and stable)
  * No timestamps, no randomness, no printing
* **Reading link (required for at least 1 challenge)**

  * **Anchor:** Effective Rust — Ch. 1 (Types) Items 1–3, 5, 9
  * **How it changes your implementation (1 line):** You must use types (`Result`, domain error enum) to make invalid transitions explicit instead of panicking or silently ignoring them.
* **What skill it builds for the project (1 line)**

  * Golden determinism testing—exactly what you’ll use to prove `--seed` reproducibility later.
