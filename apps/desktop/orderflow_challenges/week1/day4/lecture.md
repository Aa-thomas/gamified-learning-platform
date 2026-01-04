# Metrics & Invariants

## Track metrics and enforce system invariants

### Why This Matters

- Metrics provide observability into your simulator's behavior
- Invariants catch bugs early and ensure system correctness
- Well-designed metrics support both debugging and analysis

### Key Concepts

#### Core Metrics

```rust
struct Metrics {
    total_orders: u64,
    total_cancels: u64,
    total_fills: u64,
    queue_depth: u64,
}

impl Metrics {
    fn apply(&mut self, event: &Event) {
        match event {
            Event::NewOrder(_) => {
                self.total_orders += 1;
                self.queue_depth += 1;
            }
            Event::Cancel(_) => {
                self.total_cancels += 1;
                self.queue_depth -= 1;
            }
            Event::Fill(_) => {
                self.total_fills += 1;
                self.queue_depth -= 1;
            }
        }
    }
    
    fn cancel_rate(&self) -> f64 {
        if self.total_orders == 0 { 0.0 }
        else { self.total_cancels as f64 / self.total_orders as f64 }
    }
}
```

#### Key Invariants

1. **IDs are unique** and monotonically assigned
2. **fills ≤ new_orders** (can't fill more than placed)
3. **Cancel references existing open order** (or becomes Reject)
4. **Queue depth never negative**
5. **Cancel rate in [0, 1]**
6. **Regime transitions are deterministic**

### Testing Invariants

```rust
#[test]
fn invariant_queue_depth_never_negative() {
    let mut metrics = Metrics::default();
    let events = generate_test_events(100);
    
    for e in events {
        metrics.apply(&e);
        assert!(metrics.queue_depth >= 0);
    }
}
```

### Prove You Learned It

1. I can implement a Metrics struct that updates correctly on each event.
2. I can write tests that verify invariants hold across event sequences.
3. I can identify when an invariant violation indicates a bug in the core logic.
