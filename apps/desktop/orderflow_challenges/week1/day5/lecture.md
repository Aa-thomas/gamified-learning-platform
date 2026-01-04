# Integration & CLI Design

## Integrate all components and design a clean CLI

### Why This Matters

- A well-designed CLI makes your simulator usable and testable
- Integration reveals edge cases missed in unit testing
- Clean separation between core logic and I/O enables future flexibility

### Key Concepts

#### CLI Structure with `clap`

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Seed for deterministic RNG
    #[arg(long, default_value = "42")]
    seed: u64,
    
    /// Number of ticks to simulate
    #[arg(long, default_value = "1000")]
    ticks: u32,
    
    /// Output format: text or json
    #[arg(long, default_value = "text")]
    output: String,
}
```

#### Core/Shell Separation

```rust
// Core returns data, shell handles I/O
fn run_simulation(seed: u64, ticks: u32) -> SimResult {
    // Pure logic - no printing
}

fn main() {
    let args = Args::parse();
    let result = run_simulation(args.seed, args.ticks);
    
    // Shell handles output
    match args.output.as_str() {
        "json" => println!("{}", serde_json::to_string(&result).unwrap()),
        _ => println!("{}", result.to_text()),
    }
}
```

#### Determinism Proof in README

```markdown
## Reproducibility

Same seed always produces identical output:

$ cargo run -- --seed 42 --ticks 100 > run1.txt
$ cargo run -- --seed 42 --ticks 100 > run2.txt
$ diff run1.txt run2.txt  # No output = identical
```

### Integration Checklist

1. **Engine + Agents**: Agent actions flow to engine correctly
2. **Engine + Metrics**: All events update metrics
3. **Regime Transitions**: Automatic based on tick/state
4. **Trace Output**: Stable, serializable, diff-able
5. **CLI UX**: Helpful error messages, sane defaults

### Prove You Learned It

1. I can wire up clap to parse --seed and --ticks arguments.
2. I can integrate agents, engine, and metrics into a working simulation loop.
3. I can prove determinism by running twice with same seed and diffing output.
