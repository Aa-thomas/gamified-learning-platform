# Introduction to Rust

Welcome to your first Rust lesson! In this lecture, you'll learn the fundamentals of Rust programming.

## What is Rust?

Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It was created by Mozilla and first released in 2010.

## Key Features

- **Memory safety** without garbage collection
- **Zero-cost abstractions** - high-level features with no runtime overhead
- **Fearless concurrency** - write parallel code without data races
- **Great tooling** - cargo, rustfmt, clippy

## Hello World

Let's write our first Rust program:

```rust
fn main() {
    println!("Hello, world!");
}
```

The `main` function is the entry point of every Rust program. The `println!` macro prints text to the console.

## Variables

In Rust, variables are immutable by default:

```rust
fn main() {
    let x = 5;
    println!("The value of x is: {}", x);
    
    // This would cause a compile error:
    // x = 6;
}
```

To make a variable mutable, use the `mut` keyword:

```rust
fn main() {
    let mut x = 5;
    println!("The value of x is: {}", x);
    
    x = 6;  // This is allowed now
    println!("The value of x is: {}", x);
}
```

## Data Types

Rust is a statically typed language. Here are some common types:

### Integers
- `i8`, `i16`, `i32`, `i64`, `i128` (signed)
- `u8`, `u16`, `u32`, `u64`, `u128` (unsigned)

### Floating Point
- `f32`, `f64`

### Boolean
- `bool` (`true` or `false`)

### Character
- `char` (Unicode scalar value)

## Functions

Functions are declared using the `fn` keyword:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon = implicit return
}

fn main() {
    let result = add(5, 3);
    println!("5 + 3 = {}", result);
}
```

## Next Steps

Complete the quiz to test your understanding of Rust basics!
