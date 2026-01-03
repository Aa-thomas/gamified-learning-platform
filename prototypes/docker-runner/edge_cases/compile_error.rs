/// Edge Case: Compile Error
/// This student code has syntax errors that should fail compilation

pub fn fibonacci(n: u32) -> u64 {
    // Missing return type and syntax error
    let result = match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
    // Missing semicolon - compile error
    result
}

pub fn is_prime(n: u32) -> bool {
    // Type mismatch - should return bool but returns u32
    if n <= 1 {
        return n  // Missing semicolon and wrong type
    }

    // Undefined variable
    undefined_variable
}
