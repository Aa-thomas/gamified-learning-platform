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

#[derive(Debug, PartialEq)]
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

pub fn apply(state: &mut State, event: Event) -> Result<(), DomainErr> {
    match event {
        Event::New(id) => {
            state.open.push(id);
            Ok(())
        }
        Event::Cancel(id) => {
            if let Some(id) = state.open.iter().position(|&filter| filter == id) {
                state.open.remove(id);
                Ok(())
            } else {
                return Err(DomainErr::UnknownId(id));
            }
        }
    }
}

// i want to modify state. state is a vec. depending if it is new event or cancel event. cancel
// event must verify the id exists otherwise return an error
//
// inputs: State (a vector ) and Event (a pattern to match)
// outputs: No output. we must modify state inside this function
//
// i must verify state exists.
// i must match based on event
// if event::new i must add? state is a vector. ill push
// if event::cancel i must read state -> if state contains id remove  else return the DomainErr
// enum
//
// i dont know how to use match statements.
//

#[cfg(test)]
mod day1_apply_event_tests {

    use super::*;

    #[test]
    fn new_adds_to_state() {
        let mut state: State = State { open: Vec::new() };

        let event = Event::New(10);

        apply(&mut state, event).unwrap();

        let expected = 10;
        let result = state.open[0];

        println!("expected:{}\n result: {}", &expected, &result);
        assert_eq!(expected, result)
    }

    #[test]
    fn cancelling_id_removes_it() {
        let mut state: State = State { open: Vec::new() };

        for i in 0..20 {
            apply(&mut state, Event::New(i));
            println!("{:?}", &state)
        }

        let target_id = 10;

        let cancel = Event::Cancel(target_id.clone());

        apply(&mut state, cancel);

        for i in 0..20 {
            println!("{:?}", &state)
        }
        assert_eq!(state.open.contains(&target_id), false)
    }

    #[test]
    fn unknown_id_returns_error() {
        let mut state: State = State { open: Vec::new() };

        for i in 0..20 {
            apply(&mut state, Event::New(i));
            println!("{:?}", &state)
        }

        let before = state.open.clone();
        let target_id = 99;

        let cancel = Event::Cancel(target_id);

        let err = apply(&mut state, cancel).unwrap_err();
        assert_eq!(err, DomainErr::UnknownId(target_id));
        assert_eq!(state.open, before);

        for i in 0..20 {
            println!("{:?}", &state)
        }
    }
}
