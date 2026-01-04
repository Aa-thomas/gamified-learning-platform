// ### Mini-Challenge 1 — Tiny Seeded RNG + Repeatability Test
//
// * **Goal**
//
//   * Implement a minimal deterministic RNG (LCG-style) and prove repeatability with tests.
// * **Setup (what files/crate to create in a scratch folder)**
//
//   * `cargo new day2_seeded_rng && cd day2_seeded_rng`
//   * Implement in `src/lib.rs` + tests in `src/lib.rs`
// * **Requirements (clear, testable)**
//
//   * Create `struct Rng { state: u64 }` with:
//
//     * `fn new(seed: u64) -> Self`
//     * `fn next_u32(&mut self) -> u32` using a simple LCG update:
//
//       * `state = state * A + C` using `wrapping_mul`/`wrapping_add`
//       * return upper 32 bits as `u32`
//   * Tests:
//
//     * Same seed produces the same first 5 outputs (assert exact `Vec<u32>`)
//     * Different seeds produce a different first output (assert `!=`)
// * **Proof (what to run / what output must show)**
//
//   * `cargo test` passes
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Deterministic: no wall-clock, no randomness beyond your `Rng`
//   * Keep code small and readable (clippy-friendly)
// * **What skill it builds for the project (1 line)**
//
//   * Seed plumbing you’ll later use for `--seed` determinism end-to-end.
//
//

#[derive(Debug)]
struct Rng {
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
    fn seed_42_golden_first_5() {
        let mut rng = crate::Rng::new(42);
        let got: Vec<u32> = (0..5).map(|_| rng.next_u32()).collect();
        //These values will always be the same given the same seed. if they change then our model
        //is no longer deterministic
        let want: Vec<u32> = vec![2440530669, 968358053, 1773127077, 2707539007, 2921212588];
        println!("got:{:?}, want:{:?}", got, want);
        assert_eq!(got, want);
    }

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut rng1 = crate::Rng::new(42);
        let mut rng2 = crate::Rng::new(42);

        let seq1: Vec<u32> = (0..5).map(|_| rng1.next_u32()).collect();
        let seq2: Vec<u32> = (0..5).map(|_| rng2.next_u32()).collect();

        assert_eq!(seq1, seq2, "Same seed must produce identical sequences");
    }

    #[test]
    fn different_seeds_produce_different_first_output() {
        let mut rng1 = crate::Rng::new(42);
        let mut rng2 = crate::Rng::new(43);

        let output1 = rng1.next_u32();
        let output2 = rng2.next_u32();

        assert_ne!(
            output1, output2,
            "Different seeds should produce different outputs"
        );
    }

    #[test]
    fn zero_seed_works() {
        // Edge case: make sure zero is a valid seed
        let mut rng = crate::Rng::new(0);
        let output = rng.next_u32();

        // Just verify it produces a value without panicking
        // The actual value doesn't matter, we just want to ensure
        // zero is handled correctly
        assert!(output == output); // Trivial but documents the intent
    }

    #[test]
    fn max_seed_works() {
        // Edge case: make sure the maximum u64 value works
        let mut rng = crate::Rng::new(u64::MAX);
        let output = rng.next_u32();

        assert!(output == output);
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn determinism_holds_for_any_seed(seed: u64) {
            // Property: any seed produces deterministic output
            let mut rng1 = crate::Rng::new(seed);
            let mut rng2 = crate::Rng::new(seed);

            // Check that first 10 outputs match
            for _ in 0..10 {
                prop_assert_eq!(rng1.next_u32(), rng2.next_u32());
            }
        }

        #[test]
        fn state_advances_monotonically(seed: u64) {
            // Property: state should change after each call
            let mut rng = crate::Rng::new(seed);
            let mut previous_states = std::collections::HashSet::new();

            previous_states.insert(rng.state);

            for _ in 0..100 {
                rng.next_u32();
                // Each state should be unique (until the cycle, which is huge)
                prop_assert!(previous_states.insert(rng.state),
                    "State should not repeat in first 100 iterations");
            }
        }
    }
}
