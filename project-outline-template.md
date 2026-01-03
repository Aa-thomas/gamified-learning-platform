# Techdegree-Plus Project Outline Template

*A reusable generator for tasks, rubrics, quizzes, checkpoints, and portfolio artifacts.*

## Module Metadata

* **Stage / Track:** [Rust Foundations | Systems Core | Quant Evaluation | Risk/OMS | Integration/Ops]
* **Lab / Project #:** [03]
* **Title:** [In-Memory KV Store]
* **Duration:** [1–4 weeks] (default: 4-week sprint)
* **Difficulty:** [Intro | Intermediate | Advanced]
* **Flagship Repo Linkage:** [Repo A/B/C/D or “Standalone”]
* **Domain Tag:** [HFT Systems | Market Data | Matching | Backtesting | Risk | Infra]

## Hiring Signal

**Claim:** By the end, the student can:

* [Capability statement under constraints, e.g., “build a deterministic, tested KV engine with measurable performance and safe error handling”]

**Evidence artifacts (must exist):**

* `README.md` (usage + examples)
* `DESIGN.md` (tradeoffs + invariants + complexity)
* `BENCH.md` (methodology + results + regressions)
* `RUNBOOK.md` (failure modes + debug steps + repro)
* `tests/` (unit + integration + property/fuzz as applicable)
* `demo/` (scripts or commands to reproduce results)

---

# 1) Learning Objectives (Measurable)

Use **4–6** objectives. Each must map to a rubric item and a deliverable.

* **Implement:** [core feature] with [constraint]
* **Design:** [API + errors] that are [deterministic / safe / ergonomic]
* **Prove:** [invariants] using [tests/property/fuzz]
* **Measure:** [performance] with [bench method] and interpret results
* **Operate:** [debug/diagnose] using [logs/metrics/runbook]
* **Explain:** [design defense] in <N minutes using talk track

---

# 2) Lecture Themes (Concept → Practice)

Provide **3–5** themes. Each theme must connect to an exercise and a checkpoint.

1. **[Theme]:** [concept] → [how it appears in this lab] → [failure modes]
2. **[Theme]:** [tradeoffs] → [why chosen approach] → [how to justify]
3. **[Theme]:** [correctness] → [invariants + tests] → [what “proof” means here]
4. **[Theme]:** [measurement] → [bench pitfalls] → [how to trust results]
5. **[Theme]:** [domain link] → [how this maps to trading systems]

---

# 3) Core Readings (Primary Sources Preferred)

List **2–5** items max (don’t overwhelm). Include exact sections.

* **Primary:** [Official docs / book] — [sections]
* **Primary:** [Rust/Systems reference] — [chapters]
* **Applied:** [Industry post/talk] — [why it matters to this lab]
* **Optional deep dive:** [paper] — [what question it answers]

---

# 4) Discussion Prompts (Senior-style reasoning)

Provide **4** prompts that force tradeoffs, not recall.

1. **Tradeoff:** [A vs B] under [constraint: latency/correctness/ops]
2. **Failure mode:** What breaks if [assumption] is false?
3. **Design defense:** Why is your API shaped this way? What did you reject?
4. **Domain mapping:** Where does this component exist in a trading stack?

---

# 5) Pre-Project Micro-Challenges (Techdegree cadence)

Provide **6–12** short tasks. Each has: **Goal / Input / Output / Proof**.

### Micro-Challenge Format

* **MC-[#] Title**

  * **Goal:** [...]
  * **Input:** [...]
  * **Output:** [...]
  * **Proof:** [test(s) / screenshot / benchmark / doc snippet]

(Examples: “remove panic paths,” “add error context,” “property test vs reference model,” “CLI UX.”)

---

# 6) Main Project Spec

## Project: [Project Title]

### Scope

2–4 sentences describing:

* what is built
* what problem it solves
* what constraints apply (determinism, no panics, perf, etc.)
* why it matters in real systems

### Requirements

Split into **Must / Should / Could** to prevent scope creep.

**Must (required)**

* [R1]
* [R2]
* [R3]

**Should (recommended)**

* [R4]
* [R5]

**Could (stretch; pick 1–2)**

* [S1]
* [S2]

### Non-Functional Constraints (always explicit)

* **Safety:** [no panics / explicit errors / no UB]
* **Determinism:** [replayable outputs, stable ordering]
* **Performance:** [bench required; one measured improvement]
* **Observability:** [logs/metrics as appropriate]
* **Ergonomics:** [clear CLI + helpful errors]

---

# 7) Deliverables (Portfolio-Grade)

List exact paths and acceptance criteria.

* `src/lib.rs` (or core module): [what it must expose]
* `src/bin/<cli>.rs`: [commands + UX expectations]
* `README.md`: [install/run/examples/limitations]
* `DESIGN.md`: must include:

  * **Chosen approach + alternatives rejected**
  * **Invariants (bullet list)**
  * **Complexity table (big-O)**
* `BENCH.md`: must include:

  * benchmark scenarios + methodology
  * results table + interpretation
  * regression policy (how to detect future regressions)
* `RUNBOOK.md`: must include:

  * how to reproduce bugs
  * how to debug perf regressions
  * common failure modes
* `tests/`: unit + integration + property/fuzz (as applicable)
* `scripts/` or `demo/`: one-command demo instructions

---

# 8) Checkpoints (Gated Progression)

Default: **4 checkpoints** (weekly). Each checkpoint has “Pass = evidence”.

### CP1 — MVP Correctness

* **Pass criteria:** [...]
* **Evidence:** tests, CLI demo commands, no panics
* **Common fails:** [...]

### CP2 — Invariants + Deep Tests

* **Pass criteria:** [...]
* **Evidence:** property tests / fuzz targets / invariant doc
* **Common fails:** [...]

### CP3 — Measurement + Optimization

* **Pass criteria:** [...]
* **Evidence:** benches + one improvement + flamegraph note
* **Common fails:** [...]

### CP4 — Polish + Explainability

* **Pass criteria:** [...]
* **Evidence:** docs complete, talk track, runbook quality
* **Common fails:** [...]

---

# 9) Rubric (100 pts, senior-weighted)

Use these categories unless you have a strong reason to deviate.

* **Correctness & Semantics (25):** [...]
* **Error Handling & Reliability (15):** [...]
* **Architecture & Design (15):** [...]
* **Testing Depth (20):** [...]
* **Performance & Measurement (15):** [...]
* **Documentation & Explainability (10):** [...]

**Bonus (+10 max)**

* [+5] [stretch feature with tests]
* [+5] [advanced testing or ops readiness]

---

# 10) Excellence Indicators (what “exceptional” looks like)

List **6–10** measurable signals.

* Invariants are enforced by property tests with shrinking-friendly failures
* Bench methodology discusses noise + variance + regression guarding
* One optimization is justified by data, not vibes
* Errors carry actionable context without leaking internals
* CLI has clean UX + exit codes + helpful “not found” semantics
* Runbook enables someone else to debug your system in <10 minutes
* Design doc defends rejected alternatives clearly
* Determinism is explicitly tested

---

# 11) Quiz Pack (15–25 questions)

Structure it so it’s auto-generatable and balanced.

### Quiz Sections (include counts)

* **Rust & API Design (5):** [...]
* **Data Structures & Complexity (5):** [...]
* **Correctness & Invariants (5):** [...]
* **Testing Strategy (3):** [...]
* **Benchmarking & Perf (3):** [...]
* **Domain Link (2):** [...]

Each question should be tagged:

* **Type:** [Recall | Reasoning | Scenario | Debug]
* **Difficulty:** [1–3]
* **Expected answer bullets:** [2–4 bullets]

---

# 12) Interview Talk Track (mandatory)

A short script the student can practice.

* **60 seconds:** what you built + why
* **2 minutes:** design tradeoffs + invariants
* **2 minutes:** testing strategy (incl property/fuzz)
* **2 minutes:** benchmarking method + one optimization
* **1 minute:** “what I’d do next for production / concurrency”

---

# 13) Domain Link (Concrete)

2–4 sentences: where this appears in real trading systems and why it matters.

* Example mapping: [component] → [system] → [risk/perf consequence]

---

# 14) Tooling & Workflow Requirements

* Lints: [clippy rules / no unwrap policy]
* Formatting: [rustfmt]
* CI: [tests + clippy + fmt + (optional) benches]
* Commands: `just` or `cargo xtask` targets:

  * `test`, `lint`, `bench`, `demo`, `docs`

---

# 15) Starter Kit (Optional Scaffolding)

If you provide starter code, specify:

* what is prebuilt
* what students must implement
* what tests are failing initially
* what “ownership” means (no black boxes)

---

## Consistency Checklist (must pass before shipping a lab)

* [ ] Objectives map to rubric + deliverables
* [ ] Each checkpoint has evidence + pass criteria
* [ ] Invariants are written + tested
* [ ] BENCH.md includes methodology (noise, variance, regression)
* [ ] RUNBOOK.md exists and is actionable
* [ ] Quiz has scenario + reasoning questions (not only recall)
* [ ] Talk track exists and matches the project’s true design
* [ ] Scope is controlled (Must/Should/Could)

