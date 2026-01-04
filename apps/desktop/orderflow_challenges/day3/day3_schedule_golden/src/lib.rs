// ### Mini-Challenge 2 — Deterministic Scheduling + Golden “Tick Summary” (Reading Proof)
//
// * **Goal**
//
//   * Prove you can schedule multiple agents deterministically and lock down results with a golden test.
// * **Setup (what files/crate to create in a scratch folder)**
//
//   * `cargo new day3_schedule_golden && cd day3_schedule_golden`
//   * Implement in `src/lib.rs` + tests in `src/lib.rs`
// * **Requirements (clear, testable)**
//
//   * Reuse `Regime`, `Ctx`, `Action`, `Agent`, and `Rng` (copy/paste OK).
//   * Implement `fn run_tick(agents: &mut [Box<dyn Agent>], ctx: &Ctx, rng: &mut Rng) -> Vec<(u32, Action)>` that:
//
//     * Sorts agents by `id()` **every call** (or assumes input already sorted—your choice, but be explicit)
//     * Collects actions as `(agent_id, action)` preserving emission order
//   * Implement `fn fingerprint(actions: &[(u32, Action)]) -> String` that:
//
//     * Produces a stable string like: `"a1:Place(3),a2:Cancel(5),..."`
//   * Golden test:
//
//     * Fixed seed + fixed ctx + same two agents ⇒ exact fingerprint string
//   * Add a short comment near the golden test:
//
//     * `// RfR Ch.6: golden tests catch determinism regressions; this fingerprint must not drift.`
// * **Proof (what to run / what output must show)**
//
//   * `cargo test` passes
//   * Golden fingerprint assertion is byte-for-byte exact
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Deterministic ordering: sort by `AgentId`; do not rely on insertion order from an unordered source
//   * No printing in tests
// * **Reading link (required for at least 1 challenge)**
//
//   * **Anchor:** Rust for Rustaceans — Ch. 6 (Testing)
//   * **How it changes your implementation (1 line):** You must lock down deterministic scheduling behavior with a golden fingerprint so refactors can’t silently reorder actions.
// * **What skill it builds for the project (1 line)**
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Calm,
    Burst,
    CancelStorm,
}

pub struct Ctx {
    pub tick: u32,
    pub regime: Regime,
    pub open_ids: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Place(u32),
    Cancel(u32),
}

pub trait Agent {
    fn id(&self) -> u32;
    fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action>;
}

#[derive(Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Initialize with the provided seed
        // The same seed will always produce the same sequence
        Rng { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
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

    pub fn next_bool(&mut self) -> bool {
        (self.next_u32() & 1) == 1
    }
}

pub fn pick_open_id(ctx: &Ctx, rng: &mut Rng) -> Option<u32> {
    if ctx.open_ids.is_empty() {
        return None;
    }
    let idx = (rng.next_u32() as usize) % ctx.open_ids.len();
    Some(ctx.open_ids[idx])
}

fn run_tick(agents: &mut [Box<dyn Agent>], ctx: &Ctx, rng: &mut Rng) -> Vec<(u32, Action)> {
    agents.sort_by_key(|agent| agent.id());
    let mut actions: Vec<(u32, Action)> = Vec::new();
    for agent in agents {
        let action = agent.step(ctx, rng).remove(0);
        actions.push((agent.id(), action));
    }
    actions
}

fn fingerprint(actions: &[(u32, Action)]) -> String {
    let string = String::from("");
    for action in actions {
        let agent_id = action.0;
        let agent_action = action.1;
        println!("a{:?}:{:?}", agent_id, agent_action)
    }
    string
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PlaceAgent {
        id: u32,
        n: usize,
    }

    impl PlaceAgent {
        fn new(id: u32, n: usize) -> Self {
            Self { id, n }
        }
    }

    impl Agent for PlaceAgent {
        fn id(&self) -> u32 {
            self.id
        }

        fn step(&mut self, ctx: &Ctx, _rng: &mut Rng) -> Vec<Action> {
            (0..self.n)
                .map(|i| Action::Place(ctx.tick + i as u32))
                .collect()
        }
    }

    struct CancelFromOpenAgent {
        id: u32,
    }

    impl CancelFromOpenAgent {
        fn new(id: u32) -> Self {
            Self { id }
        }
    }

    impl Agent for CancelFromOpenAgent {
        fn id(&self) -> u32 {
            self.id
        }

        fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action> {
            match pick_open_id(ctx, rng) {
                Some(x) => vec![Action::Cancel(x)],
                None => vec![],
            }
        }
    }

    #[test]
    fn run_tick_sorts_by_id_and_preserves_emission_order() {
        let ctx = Ctx {
            tick: 10,
            regime: Regime::Calm,
            open_ids: vec![100, 200, 300],
        };

        // Deliberately unsorted input: id 2 then id 1
        let mut agents: Vec<Box<dyn Agent>> = vec![
            Box::new(PlaceAgent::new(2, 1)),
            Box::new(PlaceAgent::new(1, 2)),
        ];

        let mut rng = Rng::new(123);
        let got = run_tick(&mut agents, &ctx, &mut rng);

        let expected = vec![
            (1, Action::Place(10)),
            (1, Action::Place(11)),
            (2, Action::Place(10)),
        ];

        assert_eq!(got, expected);
    }

    #[test]
    fn golden_fingerprint_is_byte_for_byte_stable() {
        // RfR Ch.6: golden tests catch determinism regressions; this fingerprint must not drift.
        let ctx = Ctx {
            tick: 3,
            regime: Regime::Calm,
            open_ids: vec![5, 6, 7],
        };

        let mut agents: Vec<Box<dyn Agent>> = vec![
            Box::new(CancelFromOpenAgent::new(2)),
            Box::new(PlaceAgent::new(1, 1)),
        ];

        let mut rng = Rng::new(7);
        let actions = run_tick(&mut agents, &ctx, &mut rng);

        // With seed=7 and open_ids=[5,6,7], the first pick_open_id selects index 1 -> 6.
        let got = fingerprint(&actions);
        let expected = "a1:Place(3),a2:Cancel(6)".to_string();

        assert_eq!(got, expected);
    }
}
