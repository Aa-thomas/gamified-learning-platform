// ### Mini-Challenge 2 — Deterministic Trace Fingerprint (with Reading Proof)
//
// * **Goal**
//
//   * Prove you can produce a **stable, reproducible trace summary** from a deterministic transition function.
// * **Setup (what files/crate to create in a scratch folder)**
//
//   * `cargo new day1_trace_fingerprint && cd day1_trace_fingerprint`
//   * Logic in `src/lib.rs`, tests in `src/lib.rs`
// * **Requirements (clear, testable)**
//
//   * Reuse `Event`, `State`, `DomainErr` from Challenge 1 (copy/paste is fine).
//   * Implement `fn run_script(events: &[Event]) -> Result<String, DomainErr>` that:
//
//     * Starts from `State { open: vec![] }`
//     * Applies all events in order
//     * Returns a **fingerprint string** like: `"open=[2,5];len=2"`
//   * Add **1 golden test**:
//
//     * Given a fixed script (you choose 6–10 events), the returned fingerprint matches an exact expected string.
//   * Add a short comment at the top of `run_script` explaining why this function is “pure core” (1–2 sentences).
// * **Proof (what to run / what output must show)**
//
//   * `cargo test` passes
//   * The golden test asserts an exact, stable fingerprint string
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Don’t use `HashMap` here (stick to `Vec` so ordering is obvious and stable)
//   * No timestamps, no randomness, no printing
// * **Reading link (required for at least 1 challenge)**
//
//   * **Anchor:** Effective Rust — Ch. 1 (Types) Items 1–3, 5, 9
//   * **How it changes your implementation (1 line):** You must use types (`Result`, domain error enum) to make invalid transitions explicit instead of panicking or silently ignoring them.
// * **What skill it builds for the project (1 line)**
//
//   * Golden determinism testing—exactly what you’ll use to prove `--seed` reproducibility later.

#[derive(Debug, PartialEq, Clone)]
pub enum Event {
    New(u32),
    Cancel(u32),
}

#[derive(Debug, PartialEq)]
pub struct State {
    open: Vec<u32>,
}

#[derive(Debug, PartialEq)]
pub enum DomainErr {
    UnknownId(u32),
}

//i want to apply all events in order. events is a vector slice. i want to return a string that
//says the state of state.open.
//
//inputs: events is a vector. state.open is a vector
//outputs a string that holds details of the state of state.open
//
//i must read the event stream
//i must apply each event in deterministic order
//i must update the state of state.open
//i must use state to build a string containing "open = [open orders, total orders]; len = open
//orders"
//i must write a test showing a matching fingerprint string
//
//

fn run_script(events: &[Event]) -> Result<String, DomainErr> {
    let mut state: State = State { open: Vec::new() };

    for event in events {
        apply(&mut state, event)?;
    }

    Ok(format!("open={:?},len={}", state.open, state.open.len()))
}

pub fn apply(state: &mut State, event: &Event) -> Result<(), DomainErr> {
    match *event {
        Event::New(id) => {
            state.open.push(id);
            Ok(())
        }
        Event::Cancel(id) => {
            //here we check if the id exists first before we remove it
            if let Some(id) = state.open.iter().position(|&filter| filter == id) {
                state.open.remove(id);
                Ok(())
            } else {
                return Err(DomainErr::UnknownId(id));
            }
        }
    }
}

#[cfg(test)]
mod day1_trace_fingerprint_test {
    use super::*;

    #[test]
    fn fingerprint_matches() -> Result<(), DomainErr> {
        let events = vec![
            Event::New(2),
            Event::New(5),
            Event::Cancel(2),
            Event::New(9),
            Event::Cancel(9),
            Event::New(7),
        ];
        let result = run_script(&events)?;
        let expected = "open=[5, 7],len=2";
        assert_eq!(result, expected);
        Ok(())
    }
}
