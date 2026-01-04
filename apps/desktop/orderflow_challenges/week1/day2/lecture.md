# Determinism Plumbing: Seeded RNG & Golden Replay Tests

## Thread seeded RNG through your system without leaking nondeterminism

### Why This Matters

- Your simulator must prove: **same seed ⇒ same trace ⇒ same metrics**. That only works if randomness is **contained** and ordering is **stable**.
- The easiest determinism bugs are accidental: unordered iteration, hidden time, or "randomness in the engine."
- A good replay test makes regressions obvious: you'll catch "tiny refactors" that silently change behavior.

### Key Concepts

#### RNG Boundary Rule

RNG lives in "policy" (agent intent), not in "mechanics" (engine apply):

```rust
// GOOD: RNG only in agent step
fn agent_step(state: &State, rng: &mut Rng) -> Vec<Action> { ... }

// BAD: RNG in apply breaks determinism
fn apply(state: &mut State, e: Event, rng: &mut Rng) { ... }
```

#### Seed Threading

Pass `&mut Rng` explicitly—no globals:

```rust
struct Rng { state: u64 }

impl Rng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    
    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }
}
```

#### Stable Collections

Avoid relying on `HashMap` iteration for traces; use `Vec` + sort or `BTreeMap`:

```rust
// BAD: HashMap iteration order is not guaranteed
for (k, v) in metrics.iter() { ... }

// GOOD: Sort keys for stable output
let mut keys: Vec<_> = metrics.keys().collect();
keys.sort();
for k in keys { ... }
```

#### Golden Replay Test

Fixed seed + fixed steps ⇒ exact expected fingerprint/trace:

```rust
#[test]
fn golden_test() {
    let result = run(42, 12); // seed=42, ticks=12
    assert_eq!(result, "events=12;open_sorted=[1,3,7]");
}
```

### Discussion: HashMap Iteration

**Why `HashMap` iteration can break reproducibility:**

- `HashMap` iteration order is not a stable contract; it can change across runs/builds/platforms.
- For trace output, either **sort keys** or use an **ordered structure** (`BTreeMap`) so serialization is stable.
- Failure mode: "same seed, different printed trace" because output order drifted.

### Pure Core API Design

```rust
// apply is deterministic - same inputs always produce same outputs
fn apply(state: &mut State, event: Event) -> Result<(), DomainErr>

// agent_step is the only place randomness enters (via explicit rng)
fn agent_step(state: &State, rng: &mut Rng) -> Vec<Action>
```

### Prove You Learned It

1. I can implement a tiny seeded RNG and show two runs with the same seed generate the same first N numbers.
2. I can create a golden test that asserts an exact fingerprint string for a fixed seed + fixed steps.
3. I can cite **Rust for Rustaceans — Ch. 6 (Testing)** and explain why a golden/snapshot test is appropriate for determinism regressions.
