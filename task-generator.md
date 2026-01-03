

# Pragmatic Delivery Template

## 0) Inputs (the only things I need)

Paste these 6 blocks (short is fine):

**A. One-liner**

* Project: `[code] [name]` — build `[thing]` to achieve `[purpose]`

**B. Must / Should / Stretch**

* Must: …
* Should: …
* Stretch: …

**C. Non-negotiables**

* (examples) no panics, deterministic, tests required, docs required, bench required

**D. Deliverables (paths)**

* `README.md`, `DESIGN.md`, `RUNBOOK.md`, (optional `BENCH.md`)
* code entrypoints (lib/cli)

**E. Checkpoints**

* CP1: …
* CP2: …
* CP3: …
* CP4: …

**F. Rubric categories (optional)**

* Correctness, Reliability, Tests, Docs, Perf, Design

That’s it.

---

# 1) Output: Kanban Tickets (12–18 max)

**Ticket rules**

* Titles start with a verb.
* Each ticket has **Acceptance** + **Proof**.
* Every “Must” maps to ≥1 ticket.
* Every deliverable doc maps to ≥1 ticket.
* Stretch items are **separate** and clearly labeled.

**Ticket format**

```
[CODE-##] Title
Type: Feature | Tests | Docs | Refactor | Perf | Tooling
Priority: P0/P1/P2
Depends: CODE-##
Acceptance:
- ...
Proof:
- ...
Notes:
- (optional)
```

### Standard EPIC structure (use every time)

* **E0 Setup**
* **E1 Core Functionality**
* **E2 Reliability / Determinism**
* **E3 Tests**
* **E4 Docs**
* **E5 Demo/Release**
* **E6 Stretch (optional)**

---

# 2) Output: Issue Bodies (copy/paste)

**Issue body format**

```
## What / Why
(1–3 sentences)

## Acceptance
- [ ] ...
- [ ] ...

## Proof
- Tests:
- Demo:
- Docs:

## Notes (optional)
- Pitfalls:
- Suggested files:
```

No essays. Just executable clarity.

---

# 3) Output: PR Plan (4 checkpoint PRs + final PR)

**PR naming (consistent)**

* `CP1: MVP`
* `CP2: Correctness + Tests`
* `CP3: Perf + Bench` (only if bench is non-negotiable)
* `CP4: Docs + Polish`
* `Release: Final`

**PR template**

```
## Summary
## Linked Issues
- CODE-##
## How to Test
## Evidence
- Tests:
- Demo:
- Bench (if required):
## Risk / Tradeoffs
## Checklist
- [ ] no panics
- [ ] tests updated/added
- [ ] docs updated
- [ ] determinism (if required)
- [ ] bench updated (if required)
```

---

# 4) Output: Release Checklist + Demo Script

**Release checklist (always)**

* [ ] `cargo test`
* [ ] `cargo clippy` (or lint command)
* [ ] README has run commands
* [ ] DESIGN lists invariants + tradeoffs
* [ ] RUNBOOK has “how to reproduce bug/perf regression”
* [ ] Demo script runs start-to-finish

**Demo script format**

```
# build
# run demo
# deterministic repro (if required)
# expected output
```

---

# 5) Output: Docs Skeletons (headings only)

Generate minimal headings for:

* `README.md`
* `DESIGN.md`
* `RUNBOOK.md`
* `BENCH.md` (only if required)

---

# 6) Definition of Done

**Ticket DoD**

* passes tests
* no panics (or policy)
* acceptance met
* proof provided
* docs updated if behavior changed

**Project DoD**

* all P0/P1 tickets done
* demo script works
* docs complete
* checkpoint PRs merged

---

## Copy-paste prompt you will use every time

```
Act as a pragmatic senior quant. Generate:
1) 12–18 kanban tickets
2) full issue bodies for each ticket
3) PR plan (CP1–CP4 + Release)
4) release checklist + demo script
5) docs skeletons
6) definition of done

INPUT:
A) One-liner: ...
B) Must: ...
   Should: ...
   Stretch: ...
C) Non-negotiables: ...
D) Deliverables: ...
E) Checkpoints: ...
F) Rubric (optional): ...
```

