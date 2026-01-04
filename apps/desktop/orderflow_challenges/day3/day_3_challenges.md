
# Pre-Project Day — Agents + Regimes: Deterministic Policies, Schedules, and Invariants

## 1) Lecture Topic

* **Title (1 line):** Design an `Agent` trait that generates deterministic intent under regimes (calm → burst → cancel-storm)

* **Why this matters for the project (2–3 bullets)**

  * Your simulator needs **behavioral variety** (noise trader vs cancel bot) without turning into spaghetti.
  * Regimes are a forcing function: you’ll change rates and mixes—if the schedule isn’t explicit, determinism will drift.
  * Agents are where “policy” lives; your engine must remain a **boring state machine** to stay testable and replayable.

* **Required reading (must include exact sections)**

  * Reading 1: **[Rust in Action] — [Ch. 2 (Language foundations) + Ch. 3 (Compound data types)]** → “Takeaway rule for today”: *Use traits + enums to structure behavior and avoid conditionals scattered across the codebase.*
  * Reading 2: **[Effective Rust] — [Ch. 1 (Types) Items 1–3, 5, 9]** → “Takeaway rule for today”: *Make the contract explicit: inputs/outputs define behavior; don’t hide dependencies like RNG or “current regime.”*
  * Reading 3: **[Rust for Rustaceans] — [Ch. 1 (Foundations)]** → “Takeaway rule for today”: *Write invariants early and encode them; don’t rely on panics or “it probably won’t happen.”*

* **Key concepts (5–8 bullets)**

  * **Agent boundary:** agent decides *what to try* (intent/actions), engine decides *what happens* (events/state updates)
  * **Trait contract:** `fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action>`
  * **Regime as input:** `Regime` must be explicit in `Ctx` so behavior is deterministic
  * **Deterministic schedule:** define agent iteration order (sorted by `AgentId`) and action ordering rules
  * **Action vs Event:** `Action` is a request; engine turns it into `Event` or `Reject`
  * **Invariant-driven design:** e.g., `Cancel` only targets known open IDs; otherwise becomes deterministic reject/no-op
  * **Stable metrics:** changes in regime should predictably change counts (orders spike in burst, cancels spike in cancel-storm)
  * **No I/O in core:** core returns trace/summary; shell prints

* **Tiny demo (optional, ≤10 lines)**

```rust
trait Agent {
    fn id(&self) -> u32;
    fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action>;
}
#[derive(Clone, Copy)] enum Regime { Calm, Burst, CancelStorm }
struct Ctx { tick: u32, regime: Regime, open_ids: Vec<u32> }
```

* **Discussion prompt of the day (from project outline)**

  * **Prompt:** *Agent contract: what belongs in `Context` (inputs), and what must NOT be there to preserve determinism and testability?*
  * **What a strong answer includes (2–3 bullets)**

    * `Context` includes only **deterministic, explicit inputs**: tick, regime, snapshot of needed state, risk limits.
    * Excludes hidden dependencies: wall-clock, global RNG, printing/log handles, mutable shared state not owned by engine.
    * Mentions failure mode: “agent mutated engine state directly” → hard-to-test, nondeterministic ordering.

* **“Prove you learned it” checklist (3 bullets)**

  * I can implement an `Agent` trait with two agents that produce different action mixes under different regimes.
  * I can define and test a deterministic “agent scheduling order” rule (by `AgentId`).
  * **Reading-based proof:** I can cite **Rust for Rustaceans — Ch. 1 (Foundations)** and list 3 invariants my agent/actions must respect.

---

## 2) Quiz (5 Questions)

* **Q1) Why is it a bug (for determinism) if agents directly mutate the engine’s order state, instead of returning actions/events?**

  * **Tags:** Reasoning + Difficulty 3 + Topics: architecture, determinism, testability
  * **Answer key:**

    * Mutation order becomes dependent on call ordering and internal agent decisions.
    * Hard to replay because the “what happened” isn’t represented as an explicit event stream.
    * Actions/events form a stable, serializable boundary that supports testing and replay.

* **Q2) (Reading) Effective Rust is big on explicit contracts. What two parameters must be explicit in an agent `step()` signature to avoid “hidden nondeterminism,” and why?**

  * **Tags:** Reading + Difficulty 3 + Topics: API design, determinism, types
  * **Answer key:**

    * `ctx: &Ctx` (explicit deterministic inputs like tick/regime/open IDs).
    * `rng: &mut Rng` (the only allowed nondeterministic source, made explicit and seeded).
    * Making both explicit enables reproducible tests and prevents accidental globals.
  * **Reading anchor:** Effective Rust — Ch. 1 (Types) Items 1–3, 5, 9; applies because it emphasizes encoding behavior contracts in function signatures and types.

* **Q3) You run agents in a `Vec<Box<dyn Agent>>`. Two different runs produce different event order even with the same seed. What’s the likely cause and one deterministic fix?**

  * **Tags:** Bug-Spotting + Difficulty 4 + Topics: ordering, scheduling, determinism
  * **Answer key:**

    * Cause: iteration order is not stable (e.g., agents inserted from unordered source, or you’re sorting inconsistently).
    * Fix: assign stable `AgentId` and sort agents by `id()` each tick (or maintain a stable ordered vector).
    * Also define deterministic ordering for actions emitted within a tick (e.g., preserve emission order).

* **Q4) (Discussion) In cancel-storm, you want cancels to surge. But if the cancel bot can “see” all internal engine details, it becomes unrealistic and hard to test. What should the cancel bot be allowed to observe, and what’s the tradeoff?**

  * **Tags:** Discussion + Difficulty 4 + Topics: modeling tradeoffs, determinism, encapsulation
  * **Answer key:**

    * Allow a constrained view: list of open IDs (maybe limited sample), regime, tick, and maybe a symbol filter.
    * Disallow direct access to internal maps/queues beyond what’s needed to choose a target.
    * Tradeoff: less “optimal” bot behavior, but better encapsulation, testability, and clearer invariants.

* **Q5) (Reading) From Rust in Action’s modeling emphasis: why is `enum Regime { Calm, Burst, CancelStorm }` better than strings like `"burst"` in core logic?**

  * **Tags:** Reading + Difficulty 2 + Topics: enums, correctness, refactors
  * **Answer key:**

    * Prevents invalid states (“brust”) at compile time.
    * Enables exhaustive `match` handling for transitions/behavior.
    * Makes refactors safe: adding a new regime forces updates everywhere.
  * **Reading anchor:** Rust in Action — Ch. 2 + Ch. 3; applies because it focuses on using enums/structs to model domain states and drive control flow via `match`.

---

## 3) Mini-Challenges (2)

### Mini-Challenge 1 — Two Agents, One Context: Deterministic Action Mix

* **Goal**

  * Implement two agents that generate different deterministic actions and prove their behavior changes with regime.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day3_agents && cd day3_agents`
  * Put everything in `src/lib.rs` with tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Define:

    * `enum Regime { Calm, Burst, CancelStorm }`
    * `struct Ctx { tick: u32, regime: Regime, open_ids: Vec<u32> }`
    * `enum Action { Place(u32), Cancel(u32) }`
    * `trait Agent { fn id(&self) -> u32; fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action>; }`
  * Implement a tiny seeded `Rng` (copy from Day 2).
  * Implement **two agents**:

    * `NoiseTrader`: in `Calm` places 1 order every 2 ticks; in `Burst` places 3 orders per tick; in `CancelStorm` places 0–1.
    * `CancelBot`: in `CancelStorm` emits up to 3 cancels targeting IDs from `ctx.open_ids` (deterministically via RNG); otherwise emits 0–1.
  * Tests:

    * With `seed=7`, `tick=10`, and a fixed `open_ids`, `CancelBot` in `CancelStorm` produces **more cancels** than in `Calm`.
    * `NoiseTrader` in `Burst` produces **more places** than in `Calm`.
* **Proof (what to run / what output must show)**

  * `cargo test` passes
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic: all randomness comes from `&mut Rng`
  * Any selection from `open_ids` must be stable (don’t mutate/sort in a nondeterministic way)
* **What skill it builds for the project (1 line)**

  * Trait-based agent modeling + regime-driven behavior without contaminating the core engine.

---

### Mini-Challenge 2 — Deterministic Scheduling + Golden “Tick Summary” (Reading Proof)

* **Goal**

  * Prove you can schedule multiple agents deterministically and lock down results with a golden test.
* **Setup (what files/crate to create in a scratch folder)**

  * `cargo new day3_schedule_golden && cd day3_schedule_golden`
  * Implement in `src/lib.rs` + tests in `src/lib.rs`
* **Requirements (clear, testable)**

  * Reuse `Regime`, `Ctx`, `Action`, `Agent`, and `Rng` (copy/paste OK).
  * Implement `fn run_tick(agents: &mut [Box<dyn Agent>], ctx: &Ctx, rng: &mut Rng) -> Vec<(u32, Action)>` that:

    * Sorts agents by `id()` **every call** (or assumes input already sorted—your choice, but be explicit)
    * Collects actions as `(agent_id, action)` preserving emission order
  * Implement `fn fingerprint(actions: &[(u32, Action)]) -> String` that:

    * Produces a stable string like: `"a1:Place(3),a2:Cancel(5),..."`
  * Golden test:

    * Fixed seed + fixed ctx + same two agents ⇒ exact fingerprint string
  * Add a short comment near the golden test:

    * `// RfR Ch.6: golden tests catch determinism regressions; this fingerprint must not drift.`
* **Proof (what to run / what output must show)**

  * `cargo test` passes
  * Golden fingerprint assertion is byte-for-byte exact
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic ordering: sort by `AgentId`; do not rely on insertion order from an unordered source
  * No printing in tests
* **Reading link (required for at least 1 challenge)**

  * **Anchor:** Rust for Rustaceans — Ch. 6 (Testing)
  * **How it changes your implementation (1 line):** You must lock down deterministic scheduling behavior with a golden fingerprint so refactors can’t silently reorder actions.
* **What skill it builds for the project (1 line)**

  * Stable “fan-in” of multi-agent actions—exactly what you’ll need before you build full event throughput + regimes.
