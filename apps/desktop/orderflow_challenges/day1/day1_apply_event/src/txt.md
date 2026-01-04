// ### Mini-Challenge 1 — Enum State Machine: `apply_event` with Typed Errors
//
// * **Goal**
//
//   * Build the smallest “event → state transition” core you can test, with **no panics** and deterministic behavior.
// * **Setup (what files/crate to create in a scratch folder)**
//
//   * `cargo new day1_apply_event && cd day1_apply_event`
//   * Implement logic in `src/lib.rs` + tests in `src/lib.rs`
// * **Requirements (clear, testable)**
//
//   * Define:
//
//     * `enum Event { New(u32), Cancel(u32) }`
//     * `struct State { open: Vec<u32> }`
//     * `enum DomainErr { UnknownId(u32) }`
//   * Implement `fn apply(state: &mut State, e: Event) -> Result<(), DomainErr>`
//
//     * `New(id)` adds to `open`
//     * `Cancel(id)` removes if present, else returns `Err(DomainErr::UnknownId(id))`
//   * Add **2 tests**:
//
//     * canceling an existing id removes it
//     * canceling an unknown id returns the correct error and does not change state
// * **Proof (what to run / what output must show)**
//
//   * `cargo test` passes
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Deterministic output: tests must not depend on printing/logging
//   * Keep functions small and single-purpose
// * **What skill it builds for the project (1 line)**
//
//   * Pure, testable transition logic—the heart of the simulator engine.
//
// ---


i need to push 
