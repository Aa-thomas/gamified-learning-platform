# Deterministic Event Modeling

## Model an event-driven core with enums + `match`

### Why This Matters

- Your simulator lives or dies on **clear event/state modeling**—if types are fuzzy, invariants and tests become impossible.
- Determinism isn't a "feature later": small choices (iteration order, IDs, branching) can silently break reproducible traces.
- If you can express transitions as **pure functions**, you can test mechanics and metrics without any I/O.

### Required Reading

1. **Rust in Action — Ch. 2-3**: Focus on enums + `match`, ownership of event/state, modeling "messages"
2. **Effective Rust — Ch. 1 Items 1–3, 5, 9**: Focus on type-driven modeling, `Option/Result` transforms
3. **Rust for Rustaceans — Ch. 1**: Real project discipline with explicit invariants

### Key Concepts

#### Event Enum as Your Contract

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    NewOrder { id: u32, symbol: String, qty: u32 },
    Cancel { id: u32 },
    Fill { id: u32, qty: u32 },
}
```

#### State Struct as Single Source of Truth

```rust
struct State {
    open_orders: Vec<u32>,
    next_id: u32,
    fills: u32,
    cancels: u32,
}
```

#### Transition Function Shape

```rust
fn apply(state: &mut State, e: Event) -> Result<(), DomainErr>
```

This is the heart of your simulator—a pure function that transforms state based on events.

### Determinism by Construction

- **Deterministic IDs**: Use sequential counters or seeded generation—never "whatever happened to come first"
- **Deterministic ordering**: Use `Vec`/sorting, or `BTreeMap`—don't rely on `HashMap` iteration
- **Invariants**: Properties that must always hold (e.g., cancels only cancel open orders)
- **Errors vs events**: When to model failures as `Reject` events vs returning `Err`
- **No unwrap/expect in core**: Core returns typed errors; shell decides how to print/exit

### Example: Minimal Apply Function

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event { New(u32), Cancel(u32) }

#[derive(Debug)]
enum DomainErr { UnknownId(u32) }

fn apply(open: &mut Vec<u32>, e: Event) -> Result<(), DomainErr> {
    match e {
        Event::New(id) => { 
            open.push(id); 
            Ok(()) 
        }
        Event::Cancel(id) => {
            open.iter()
                .position(|&x| x == id)
                .map(|i| { open.remove(i); })
                .ok_or(DomainErr::UnknownId(id))
        }
    }
}
```

### Discussion: Seed Plumbing

**Where is randomness allowed vs forbidden?**

- Randomness is **allowed only at controlled boundaries** (e.g., agent intent generation) and always via a seeded RNG passed in.
- Engine/state transitions must be **pure given (state, event)**; output formatting must not affect state or ordering.
- Identifies failure mode: "accidental randomness" via unordered iteration, timestamps, or sampling wall-clock.

### Prove You Learned It

1. I can write an `Event` enum + `match`-based `apply()` that returns `Result` (no panics) and has at least **one invariant test**.
2. I can explain why unordered iteration can break reproducible traces and how I avoid it.
3. I can cite **Effective Rust — Ch. 1 Items 1–3, 5, 9** and point to one place where a stronger type prevented an invalid state.
