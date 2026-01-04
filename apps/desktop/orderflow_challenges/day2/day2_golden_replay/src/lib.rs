// ### Mini-Challenge 2 — Golden Replay Fingerprint with Stable Ordering (Reading Proof)
//
// * **Goal**
//
//   * Build a tiny “agent generates events using RNG → engine applies” pipeline and lock it down with a golden test.
// * **Setup (what files/crate to create in a scratch folder)**
//
//   * `cargo new day2_golden_replay && cd day2_golden_replay`
//   * Implement in `src/lib.rs` + tests in `src/lib.rs`
// * **Requirements (clear, testable)**
//
//   * Reuse your `Rng` from Challenge 1 (copy/paste OK).
//   * Define:
//
//     * `enum Event { New(u32), Cancel(u32) }`
//     * `struct State { open: Vec<u32> }`
//     * `fn apply(state: &mut State, e: Event) -> Result<(), &'static str>` (no panics)
//   * Implement `fn agent_step(rng: &mut Rng, tick: u32) -> Event`:
//
//     * If `tick % 3 == 0` → `New(next_u32 % 10)`
//     * Else → `Cancel(next_u32 % 10)`
//   * Implement `fn run(seed: u64, ticks: u32) -> Result<String, &'static str>`:
//
//     * Runs ticks, collects events in a `Vec<Event>` (in order), applies them
//     * Returns a fingerprint string that is **stable**:
//
//       * include `events=<count>;open_sorted=[...]`
//       * IMPORTANT: `open_sorted` must be sorted before formatting
//   * Add a golden test for `seed=42, ticks=12` asserting the exact fingerprint string.
//   * Add a short comment citing the reading: one sentence like
//
//     * `// RfR Ch.6: golden tests lock down behavior across refactors; we assert an exact fingerprint.`
// * **Proof (what to run / what output must show)**
//
//   * `cargo test` passes
//   * Golden assertion compares to an exact string (byte-for-byte)
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Deterministic output (sorting before formatting)
//   * No `HashMap` iteration for anything you serialize
// * **Reading link (required for at least 1 challenge)**
//
//   * **Anchor:** Rust for Rustaceans — Ch. 6 (Testing)
//   * **How it changes your implementation (1 line):** You must write a golden test that asserts an exact, stable output so determinism regressions are caught immediately.
// * **What skill it builds for the project (1 line)**
//
//   * End-to-end determinism proof pattern: seed → events → apply → stable serialized summary.
//

use std::string;

// i want to take a rng(u64), and based on the tick (u64)
//     - if tick % 3 -> i will emit new order event
//     - else -> i will emit cancel event
//
//inputs: Rng (u32), tick (u64)
//outputs: Event (new) or (cancel)
//
//i must match based on tich % 3
//i must create the id by using rng.next_u32() % 10
//if condition true create Event::New(id)
//else condition false create Event::Cancel(id)
//
//
pub fn agent_step(rng: &mut Rng, tick: u32) -> Event {
    let id = rng.next_u32() % 10;
    if tick % 3 == 0 {
        Event::New(id)
    } else {
        Event::Cancel(id)
    }
}

// i want to create a deterministic sequence of events from seed. i will run for ticks steps
//
//inputs: seed(u64), ticks(u32)
//outputs: a string that holds the state: events=<count> , open_sorted=[]
//variables: state (Vec<u32>), agent_step(rng: &mut Rng, tick:u32)
//
//i must create Events Vector
//i must loop through ticks
//i must use agent step func
//  inside i must use seed, and the current tick "i"
//i must add the result of agent_step and store it in Events
//after the loop
// i must compute "events=<count>". (events total = total number of ticks)
// i must sort state.open
// i must compute "sorted_open". (sorted_open = state.open.sort())
// i return a string with the computed values
pub fn run(seed: u64, ticks: u32) -> Result<String, &'static str> {
    let mut state: State = State {
        open: Vec::with_capacity(ticks as usize),
    };
    let mut events = Vec::with_capacity(ticks as usize);
    let mut rng: Rng = Rng { state: seed };

    for tick in 0..ticks {
        let event = agent_step(&mut rng, tick);
        events.push(event.clone());
        match apply(&mut state, event) {
            Ok(()) => continue,
            Err(DomainErr::UnknownId(_)) => continue,
        }
    }

    let count = events.len();
    let mut sorted_open = state.open.clone();
    sorted_open.sort();
    let msg = format!("events={count};open_sorted={:?}", sorted_open);
    println!("{}", msg);
    Ok(msg)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

#[derive(Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        // Initialize with the provided seed
        // The same seed will always produce the same sequence
        Rng { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        // These are common LCG constants from Knuth's MMIX
        // They're chosen to have good mathematical properties
        const A: u64 = 6364136223846793005;
        const C: u64 = 1442695040888963407;

        // Update our state using the LCG formula
        // wrapping_mul and wrapping_add allow intentional overflow
        self.state = self.state.wrapping_mul(A).wrapping_add(C);

        // Return the upper 32 bits, which have better randomness properties
        // The shift >> 32 moves the high bits down, and 'as u32' keeps just those bits
        (self.state >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_replay_fingerprint_seed42_ticks12() {
        // RfR Ch.6: golden tests lock down behavior across refactors; we assert an exact fingerprint.
        let fp = match run(42, 12) {
            Ok(s) => s,
            Err(e) => panic!("run(42, 12) returned Err: {:?}", e),
        };
        assert_eq!(fp, "events=12;open_sorted=[7, 8]");
    }
}
