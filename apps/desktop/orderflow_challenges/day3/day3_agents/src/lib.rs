// ### Mini-Challenge 1 — Two Agents, One Context: Deterministic Action Mix
//
// * **Goal**
//
//   * Implement two agents that generate different deterministic actions and prove their behavior changes with regime.
// * **Setup (what files/crate to create in a scratch folder)**
//
//   * `cargo new day3_agents && cd day3_agents`
//   * Put everything in `src/lib.rs` with tests in `src/lib.rs`
// * **Requirements (clear, testable)**
//
//   * Define:
//
//     * `enum Regime { Calm, Burst, CancelStorm }`
//     * `struct Ctx { tick: u32, regime: Regime, open_ids: Vec<u32> }`
//     * `enum Action { Place(u32), Cancel(u32) }`
//     * `trait Agent { fn id(&self) -> u32; fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action>; }`
//   * Implement a tiny seeded `Rng` (copy from Day 2).
//   * Implement **two agents**:
//
//     * `NoiseTrader`: in `Calm` places 1 order every 2 ticks; in `Burst` places 3 orders per tick; in `CancelStorm` places 0–1.
//     * `CancelBot`: in `CancelStorm` emits up to 3 cancels targeting IDs from `ctx.open_ids` (deterministically via RNG); otherwise emits 0–1.
// Tests
//
// Using the same seed (e.g., seed = 7) and a fixed non-empty open_ids,
// run each agent for a window of ticks (e.g., 0..50) and compare totals:
//
// CancelBot: total cancels in CancelStorm over the window is strictly greater than total cancels in Calm.
//
// NoiseTrader: total places in Burst over the window is strictly greater than total places in Calm.
// // * **Proof (what to run / what output must show)**
// //
// //   * `cargo test` passes
// // * **Guardrails**
// //
// //   * No `unwrap/expect` in `src/lib.rs`
// //   * Deterministic: all randomness comes from `&mut Rng`
// //   * Any selection from `open_ids` must be stable (don’t mutate/sort in a nondeterministic way)
// // * **What skill it builds for the project (1 line)**
// //
// //   * Trait-based agent modeling + regime-driven behavior without contaminating the core engine.
// //
// //
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regime {
    Calm,
    Burst,
    CancelStorm,
}

pub struct Ctx {
    pub tick: u32,
    pub regime: Regime,
    pub open_ids: Vec<u32>,
}

#[derive(Debug)]
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

//ii need to implement the agent trait on some type of struct. that struct can be either noisetrader
//or cancelbot. Eech must emit a placeorder or cancel depending on ticks and regime.
//
//inputs: Agent inputs, ticks, regimes
//outputs: placeorder or cancel
//
//There must be som sort of struct that implements Agent
//there must be a function that returns an id.
//there must be a function that implements intent (step)
//i must derive intent from ticks and regime
//i must get ticks and regime from ctx.
//use provided info to calculate regime.
//i must create a golden test
//

// =============================================================================
// NoiseTrader Agent
// =============================================================================

pub struct NoiseTrader {
    id: u32,
}

impl NoiseTrader {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl Agent for NoiseTrader {
    fn id(&self) -> u32 {
        self.id
    }

    fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();
        match ctx.regime {
            Regime::Calm => {
                if ctx.tick % 2 == 0 {
                    actions.push(Action::Place(ctx.tick))
                }
            }
            Regime::Burst => {
                for _ in 0..3 {
                    actions.push(Action::Place(ctx.tick));
                }
            }
            Regime::CancelStorm => {
                if rng.next_bool() {
                    actions.push(Action::Place(ctx.tick));
                }
            }
        }
        actions
    }
}

// =============================================================================
// CancelBot Agent
// =============================================================================
pub struct CancelBot {
    id: u32,
}

impl CancelBot {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl Agent for CancelBot {
    fn id(&self) -> u32 {
        self.id
    }

    fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();
        match ctx.regime {
            Regime::CancelStorm => {
                let k = (rng.next_u32() % 4) as usize; // 0..=3
                for _ in 0..k {
                    if let Some(id) = pick_open_id(ctx, rng) {
                        actions.push(Action::Cancel(id));
                    }
                }
            }
            _ => {
                if rng.next_bool() {
                    if let Some(id) = pick_open_id(ctx, rng) {
                        actions.push(Action::Cancel(id));
                    }
                }
            }
        }
        actions
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn count_cancels(actions: &[Action]) -> usize {
        actions
            .iter()
            .filter(|a| matches!(a, Action::Cancel(_)))
            .count()
    }

    fn count_places(actions: &[Action]) -> usize {
        actions
            .iter()
            .filter(|a| matches!(a, Action::Place(_)))
            .count()
    }

    #[test]
    fn cancelbot_more_cancels_in_cancelstorm_than_calm_over_window() {
        let open_ids = vec![101, 102, 103, 104, 105];
        let seed = 7_u64;

        // Use independent RNG streams so we’re not coupling the regimes via shared RNG state.
        let mut rng_calm = Rng::new(seed);
        let mut rng_storm = Rng::new(seed);

        let mut bot_calm = CancelBot::new(2);
        let mut bot_storm = CancelBot::new(2);

        let mut calm_total = 0usize;
        let mut storm_total = 0usize;

        for tick in 0..50u32 {
            let ctx_calm = Ctx {
                tick,
                regime: Regime::Calm,
                open_ids: open_ids.clone(),
            };
            let ctx_storm = Ctx {
                tick,
                regime: Regime::CancelStorm,
                open_ids: open_ids.clone(),
            };

            calm_total += count_cancels(&bot_calm.step(&ctx_calm, &mut rng_calm));
            storm_total += count_cancels(&bot_storm.step(&ctx_storm, &mut rng_storm));
        }

        assert!(
            storm_total > calm_total,
            "storm_total={storm_total}, calm_total={calm_total}"
        );
    }

    #[test]
    fn noisetrader_more_places_in_burst_than_calm_over_window() {
        let seed = 7_u64;

        let mut rng_calm = Rng::new(seed);
        let mut rng_burst = Rng::new(seed);

        let mut nt_calm = NoiseTrader::new(1);
        let mut nt_burst = NoiseTrader::new(1);

        let mut calm_total = 0usize;
        let mut burst_total = 0usize;

        for tick in 0..50u32 {
            let ctx_calm = Ctx {
                tick,
                regime: Regime::Calm,
                open_ids: vec![],
            };
            let ctx_burst = Ctx {
                tick,
                regime: Regime::Burst,
                open_ids: vec![],
            };

            calm_total += count_places(&nt_calm.step(&ctx_calm, &mut rng_calm));
            burst_total += count_places(&nt_burst.step(&ctx_burst, &mut rng_burst));
        }

        assert!(
            burst_total > calm_total,
            "burst_total={burst_total}, calm_total={calm_total}"
        );
    }
}
