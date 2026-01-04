# Threading a Seeded RNG Through "Agent Intent → Events" Without Leaking Nondeterminism into the Core

**Course:** Advanced Quantitative Systems Design in Rust  
**Level:** Upper-division undergraduate / Graduate  
**Duration:** 90 minutes  
**Prerequisites:** Intermediate Rust, basic understanding of testing, familiarity with state machines

---

## Why This Matters for Your Project

Before we dive into the technical mechanics, let me tell you why this lecture could save you weeks of debugging pain. Imagine you've built a beautiful agent-based simulation—perhaps modeling trading strategies, epidemic spread, or traffic patterns. You run it once and get interesting results. You run it again with the same parameters, and you get different results. Is this a feature or a bug? Without determinism, you genuinely cannot tell.

Your simulator must satisfy a fundamental contract: **same seed implies same trace implies same metrics**. This property only holds if randomness is properly contained and execution ordering remains stable across runs. Think of determinism as a reproducibility guarantee—it's the difference between a scientific instrument that gives consistent readings and one that drifts unpredictably.

The most insidious determinism bugs are the accidental ones. They creep in through unordered hash map iteration, hidden system time dependencies, or randomness that accidentally leaked into your core simulation engine. These bugs are especially painful because they manifest intermittently and disappear when you try to debug them.

A well-designed replay test makes regressions immediately obvious. When you refactor code that you believed was purely structural, and suddenly your golden test fails, you've just caught a behavior change that would have been nearly impossible to detect otherwise. This is defensive programming at its finest—we're making entire classes of bugs impossible by construction.

---

## Required Reading and Core Principles

Let me frame our three required readings not as homework assignments, but as architectural philosophy that will guide every design decision we make today.

### Reading 1: Effective Rust — Chapter 1 (Types), Items 1–3, 5, 9

The first reading teaches us to make nondeterminism impossible by design. Rust's type system is not just for memory safety—it's a tool for encoding contracts. When you see a function signature like `fn process(input: Data) -> Result<Output, Error>`, you're seeing explicit documentation of where randomness, errors, and side effects can occur. Our takeaway rule for today is this: **push nondeterminism to the edges of your system and encode all contracts in types**. Use `Result` for operations that can fail, use explicit input and output parameters rather than hidden global state, and make the RNG an explicit parameter that travels through your call stack like a passport.

### Reading 2: Rust for Rustaceans — Chapter 6 (Testing)

Jon Gjengset's testing chapter introduces us to the concept of golden tests, sometimes called snapshot tests. These tests lock down exact behavior by comparing outputs against known-good references. Our takeaway rule is: **prefer tests that detect behavior changes when refactors could silently alter outputs**. In a deterministic simulation, a golden test might capture the exact sequence of events over one thousand ticks and store it as a fingerprint. Any change to that fingerprint—even one triggered by a seemingly innocent code reorganization—becomes immediately visible.

### Reading 3: Rust in Action — Chapter 2 (Language Foundations) and Chapter 3 (Compound Data Types)

Tim McNamara's chapters on language foundations remind us that explicitness trumps cleverness. Our takeaway rule: **use explicit data structures and clear control flow to keep behavior understandable and reproducible**. When you use `match` statements instead of complex boolean logic, when you iterate with explicit `for` loops instead of clever iterator chains, and when you choose data structures based on ordering guarantees rather than just performance, you're building systems that behave the same way every time.

---

## The Architecture of Deterministic Simulation

Now let's build up the conceptual model that will govern our entire design. I want you to think of your simulation as having two distinct zones with a very clear boundary between them.

### The RNG Boundary Rule

Randomness lives in policy code—the code that decides agent intent—but never in mechanics code—the code that applies events to world state. This is perhaps the most important architectural decision you'll make. Let me illustrate why with a cautionary tale.

Imagine you're simulating a market where traders submit orders. You might be tempted to write an `apply_order` function that checks inventory and, if there's not enough stock, randomly decides whether to partially fill or reject the order entirely. This seems convenient—the randomness is localized right where the decision happens. But now your core simulation engine contains randomness, which means testing becomes a nightmare. How do you write a unit test for "process this order" when the outcome is nondeterministic? How do you replay a specific scenario when the engine itself is rolling dice?

The correct design moves that randomness up into the agent layer. The agent's decision-making code, which has access to the RNG, generates an intent: "submit order for one hundred shares." The agent might even consult the RNG to decide between different order sizes. But once that intent becomes an event—`OrderSubmitted { trader_id: 42, quantity: 100 }`—the event application must be completely deterministic. The engine doesn't get to roll dice about whether the order succeeds; it either succeeds deterministically based on current state, or it fails deterministically with a clear rejection event.

### Seed Threading: No Globals Allowed

Global random number generators are convenient, seductive, and disastrous for determinism. The moment you write `rand::thread_rng().gen()` somewhere in your codebase, you've introduced a source of nondeterminism that doesn't appear in your function signatures and can't be controlled by tests.

Instead, we thread our RNG explicitly through the call chain. The pattern looks like this: your simulation's main loop owns a mutable RNG state. When it's time for an agent to make a decision, you pass `&mut rng` to the agent's step function. The agent uses the RNG to make its random choices, then returns deterministic events. Those events flow into the engine, which applies them without any access to the RNG at all.

This explicitness has a beautiful property: you can look at any function signature and immediately know whether it can introduce randomness. If a function doesn't take an RNG parameter, it cannot introduce nondeterminism. This is what we mean by encoding contracts in types—the type system enforces your architectural boundaries.

### Deterministic Schedules: Stable Ordering Matters

Here's a subtle bug that catches many simulation designers: you have five agents that all need to act in the same tick. You store them in a hash map keyed by agent ID, iterate over the hash map, and process each agent's action. Everything seems fine until one day the output changes, even though you used the same seed.

The problem is that hash map iteration order is explicitly unspecified. It can vary between runs, between Rust versions, even between debug and release builds. If Agent Three's action affects Agent Seven's available options, and they switch order, your simulation's behavior changes even though no logic changed.

The solution is to impose a stable ordering on concurrent operations. If multiple agents act in the same tick, sort them by agent ID before processing. If multiple events arrive simultaneously, process them in a defined order—perhaps event type priority, then timestamp, then a tiebreaker field. The specific order usually doesn't matter; what matters is that it's consistent and explicit.

### Stable Collections: Choose Data Structures Carefully

This connects directly to our discussion question, but let me preview the principle now. Not all collection types are created equal for deterministic systems. A `HashMap` is fast but unordered. A `BTreeMap` maintains sorted order at the cost of logarithmic operations instead of constant-time. A `Vec` requires manual sorting but gives you complete control over ordering.

For internal state where performance matters and ordering doesn't affect behavior, hash maps are fine. But for anything that affects execution order, output format, or trace generation, you need stable ordering. Spend the extra CPU cycles on a sort if necessary—the debugging time you save will be worth far more.

### Golden Replay Tests: Your Regression Safety Net

Let me describe what a proper replay test looks like in practice. You create a test that initializes your simulation with a fixed seed—let's say forty-two. You run exactly one thousand ticks. At the end, you don't just check high-level metrics like "average price was roughly three hundred." Instead, you compute a fingerprint of the entire event trace—perhaps a hash of all events in order, or a serialized representation. You assert that this fingerprint exactly matches a known good value.

Now, months later, you refactor some code. Maybe you switch from `Vec<Agent>` to `HashMap<AgentId, Agent>` for faster lookups. You run your test suite, and the golden test fails. The fingerprint doesn't match. You've just discovered that your refactor changed behavior—perhaps because iteration order changed. Without this test, that behavior change would have silently shipped to production.

### Pure Core API: Drawing Clean Lines

This is where architecture meets implementation. Your simulation should expose two types of operations. Pure core operations look like `apply(state, event) -> Result<State, Error>`. Given the same state and event, they always produce the same result. There's no hidden randomness, no system time, no network calls—just pure computation.

Then you have the policy layer: `agent_step(state, &mut rng) -> Vec<Event>`. This is where randomness is allowed. The agent observes the state, consults its random number generator to make decisions, and emits events. But once those events are created, they're deterministic data that flows into your pure core.

This separation has profound testing benefits. You can unit test your core engine without any randomness at all—just create events by hand and verify they're applied correctly. You can test your agent logic by providing a mock RNG that returns predetermined values. And you can integration test the whole system by providing a seeded RNG and verifying exact replay behavior.

### Error Strategy: Explicit Failures, Never Silent

In a deterministic system, every edge case must be handled explicitly and consistently. If an agent tries to perform an invalid action—like trading more shares than they own—what happens? 

The wrong answer is "silently do nothing and log a warning." That's nondeterministic because log processing is often non-deterministic, and it makes replay tests harder because you can't distinguish between "the event was processed and had no effect" and "the event was invalid and was rejected."

The right answer is to generate an explicit rejection event: `OrderRejected { reason: InsufficientFunds }`. This event becomes part of the trace, can be tested for, and makes the simulation's behavior completely explicit. You might choose to have `apply` return `Result<State, Error>` for true errors, while invalid-but-recoverable situations generate explicit rejection events that are part of normal execution.

### The Clippy Mindset: Simplicity as Strategy

Rust's Clippy linter often suggests making code more explicit rather than more clever. This aligns perfectly with determinism goals. Instead of a complex iterator chain with filters and flat maps, write a simple for loop where you can see exactly what happens in what order. Instead of implicit conversions and type coercion, write explicit transformations. Every line of code should be obviously correct, not cleverly correct.

---

## A Tiny Demonstration

Let me show you the simplest possible seeded RNG—a Linear Congruential Generator. This isn't cryptographically secure or particularly high quality, but it's deterministic and educational:

```rust
fn pick_id(rng: &mut u64) -> u32 {
    // LCG: multiply by a prime, add an odd number
    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
    // Return the high 32 bits (better randomness properties)
    (*rng >> 32) as u32
}
```

Notice three things. First, the RNG state is passed as a mutable reference—we're explicitly threading it through. Second, the transformation is completely deterministic—the same input state always produces the same output. Third, we mutate the state in place and return a value, making the data flow explicit.

You could use this in an agent like this:

```rust
fn agent_choose_action(state: &World, rng: &mut u64) -> Action {
    let choice = pick_id(rng) % 3;
    match choice {
        0 => Action::Buy,
        1 => Action::Sell,
        _ => Action::Hold,
    }
}
```

Because `pick_id` is deterministic and the match statement has stable control flow, calling `agent_choose_action` with the same world state and RNG state will always produce the same action.

---

## Discussion Prompt: HashMap Iteration and Reproducibility

Now let's dig into today's core discussion question, which connects directly to real bugs you'll encounter. Why can `HashMap` iteration break reproducibility, and when should you prefer `BTreeMap` or explicit sorting?

A strong answer recognizes three levels of the problem. First, at the specification level: Rust's `HashMap` explicitly does not guarantee iteration order as part of its API contract. The order can vary between runs, between different Rust versions, between platforms, even between debug and release builds. This is by design—it allows the implementation to optimize for performance.

Second, at the failure mode level: imagine you're generating a trace of your simulation for debugging. You iterate through all agents in a hash map and print their states. The printed order differs between runs, even with the same seed. Now you can't diff trace files to detect regressions. Or worse, imagine the order of iteration affects which agent processes their action first, which affects what the next agent sees. Now your simulation's behavior itself has become non-deterministic, despite using a seeded RNG.

Third, at the solution level: you have options depending on your needs. For trace output where you just need stability, sort the keys before iterating: `hash_map.keys().sorted().for_each(...)`. For cases where you need inherent ordering, use `BTreeMap` which maintains sorted order automatically. For agent processing where order affects behavior, maintain agents in a `Vec` and sort by ID before each processing round. Each solution has different performance characteristics—sorting a vector is O(n log n), BTreeMap operations are O(log n), HashMap is O(1) but unordered—so choose based on your actual constraints.

The key insight is that non-determinism isn't always where you expect it. You might spend hours debugging your RNG seeding logic when the real problem is that your output code uses hash map iteration, making identical runs look different when serialized.

---

## Proving You've Learned It

Let me give you three concrete challenges that will demonstrate mastery of these concepts.

**Challenge One: Build and verify a tiny seeded RNG.** Implement a simple RNG—the LCG from earlier is fine, or use the `rand` crate's `StdRng` with `SeedableRng`. Create a test that seeds two RNG instances with the same value, generates the first ten numbers from each, and asserts they're identical. Then modify the seed slightly and verify the sequences differ. This exercise proves you understand that determinism requires explicit seeding and that the same seed guarantees the same sequence.

**Challenge Two: Create a golden replay test.** Build a minimal simulation—even just a single agent that picks random numbers and emits them as events. Run it for a fixed number of steps with a fixed seed. Compute a fingerprint of the output—maybe hash the event sequence, maybe serialize it to JSON and hash that. Store this fingerprint as your golden value. Now refactor your code without changing logic and verify the test still passes. Then intentionally break determinism—maybe add hash map iteration somewhere—and verify the test catches it.

**Challenge Three: Cite and apply testing philosophy.** This one requires synthesis. Write a paragraph explaining why golden tests from Rust for Rustaceans Chapter Six are particularly appropriate for determinism regressions. A strong answer mentions that golden tests capture exact behavior rather than just properties, which is precisely what you need when "any change in output" constitutes a regression. Connect this to the idea that refactoring should preserve behavior, and that without golden tests, you have no way to verify that preservation happened. Mention that the failure mode—a changed fingerprint—gives you a clear signal that something meaningful changed, even if that change was unintentional.

---

## Practical Implementation Patterns

Let me close with some concrete patterns you can apply immediately in your projects.

### Pattern One: The RNG Wrapper

Instead of passing raw RNG types through your code, wrap them in a domain-specific type:

```rust
struct SimRng(StdRng);

impl SimRng {
    fn new(seed: u64) -> Self {
        SimRng(StdRng::seed_from_u64(seed))
    }
    
    fn pick_agent(&mut self, count: usize) -> usize {
        self.0.gen_range(0..count)
    }
    
    fn coin_flip(&mut self, probability: f64) -> bool {
        self.0.gen_bool(probability)
    }
}
```

This gives you two benefits: you can add domain-specific random operations with clear names, and you can later swap out the underlying RNG implementation without changing call sites.

### Pattern Two: The Event Journal

Keep a complete ordered log of all events:

```rust
struct Simulation {
    state: World,
    events: Vec<Event>,
    tick: u64,
}

impl Simulation {
    fn apply(&mut self, event: Event) -> Result<(), Error> {
        self.state.apply(&event)?;
        self.events.push(event);
        Ok(())
    }
    
    fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for event in &self.events {
            event.hash(&mut hasher);
        }
        hasher.finish()
    }
}
```

The event journal gives you complete observability and makes golden tests trivial to write.

### Pattern Three: The Deterministic Scheduler

When multiple agents act per tick:

```rust
fn process_tick(agents: &mut [Agent], world: &World, rng: &mut SimRng) -> Vec<Event> {
    // Sort agents by ID for stable processing order
    agents.sort_by_key(|a| a.id);
    
    let mut events = Vec::new();
    for agent in agents {
        let agent_events = agent.step(world, rng);
        events.extend(agent_events);
    }
    
    // Events are returned in deterministic order
    events
}
```

Notice that we sort agents before processing, ensuring that Agent Three always goes before Agent Seven, regardless of how they're stored in memory.

---

## Closing Thoughts

Determinism is not about limiting randomness—it's about controlling where randomness lives in your architecture. A well-designed deterministic simulation can model incredibly complex stochastic processes while remaining perfectly reproducible. The techniques we've covered today—explicit RNG threading, stable ordering, pure core APIs, and golden tests—form a defensive perimeter around your simulation's determinism guarantees.

When you leave this lecture today, I want you to think about every piece of state in your system and ask: is this state deterministically derived from my inputs, or could it vary between runs? Make nondeterminism visible, push it to the edges, and test your determinism assumptions aggressively. Your future self, debugging a production issue by replaying a logged seed, will thank you.

Remember: in a deterministic system, every behavior change leaves evidence. That's not a constraint—it's a superpower.

---

## Additional Resources

- **"Deterministic Simulation Testing" by Kyle Kingsbury** - explores how databases use deterministic simulation for testing
- **The `quickcheck` and `proptest` crates** - property-based testing frameworks that integrate well with seeded RNGs
- **"Debugging with Deterministic Replay" from Microsoft Research** - academic perspective on why reproducibility matters for complex systems

## Homework Assignment

Build a simple two-agent market simulation where agents randomly buy or sell. Implement golden tests for three different seeds. Then intentionally introduce three different forms of nondeterminism (hash map iteration, system time, unseeded RNG) one at a time, and demonstrate that your golden tests catch each one. Write a brief report explaining how you detected and fixed each issue.
