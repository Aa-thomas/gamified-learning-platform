# Agent Trait Design: Deterministic Policies & Regimes

## Design an Agent trait that generates deterministic intent under different regimes

### Why This Matters

- Your simulator needs **behavioral variety** (noise trader vs cancel bot) without turning into spaghetti.
- Regimes are a forcing function: you'll change rates and mixes—if the schedule isn't explicit, determinism will drift.
- Agents are where "policy" lives; your engine must remain a **boring state machine** to stay testable and replayable.

### Key Concepts

#### Agent Boundary

Agent decides *what to try* (intent/actions), engine decides *what happens* (events/state updates):

```rust
trait Agent {
    fn id(&self) -> u32;
    fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action>;
}
```

#### Regime as Explicit Input

```rust
#[derive(Clone, Copy)]
enum Regime { Calm, Burst, CancelStorm }

struct Ctx {
    tick: u32,
    regime: Regime,
    open_ids: Vec<u32>,
}
```

#### Action vs Event

- `Action` is a request from an agent
- Engine turns it into `Event` or `Reject`

```rust
enum Action { Place(u32), Cancel(u32) }
enum Event { NewOrder(u32), Cancelled(u32), Rejected(u32) }
```

#### Deterministic Schedule

Define agent iteration order (sorted by `AgentId`) and action ordering rules:

```rust
fn run_tick(agents: &mut [Box<dyn Agent>], ctx: &Ctx, rng: &mut Rng) -> Vec<(u32, Action)> {
    agents.sort_by_key(|a| a.id()); // Stable order!
    agents.iter_mut()
        .flat_map(|a| {
            let id = a.id();
            a.step(ctx, rng).into_iter().map(move |action| (id, action))
        })
        .collect()
}
```

### Example Agents

```rust
struct NoiseTrader { id: u32 }

impl Agent for NoiseTrader {
    fn id(&self) -> u32 { self.id }
    
    fn step(&mut self, ctx: &Ctx, rng: &mut Rng) -> Vec<Action> {
        match ctx.regime {
            Regime::Calm => if ctx.tick % 2 == 0 { vec![Action::Place(rng.next_u32())] } else { vec![] },
            Regime::Burst => vec![Action::Place(rng.next_u32()); 3],
            Regime::CancelStorm => vec![], // quiet during storm
        }
    }
}
```

### Invariant-Driven Design

- `Cancel` only targets known open IDs; otherwise becomes deterministic reject/no-op
- Changes in regime should predictably change counts (orders spike in burst, cancels spike in cancel-storm)
- No I/O in core: core returns trace/summary; shell prints

### Prove You Learned It

1. I can implement an `Agent` trait with two agents that produce different action mixes under different regimes.
2. I can define and test a deterministic "agent scheduling order" rule (by `AgentId`).
3. I can cite **Rust for Rustaceans — Ch. 1** and list 3 invariants my agent/actions must respect.
