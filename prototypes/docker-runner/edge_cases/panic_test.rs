/// Edge Case: Panic in Test
/// This student code causes a panic during test execution

pub fn fibonacci(n: u32) -> u64 {
    // Intentionally panic on certain inputs
    if n > 10 {
        panic!("Fibonacci not implemented for n > 10!");
    }

    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn is_prime(n: u32) -> bool {
    // Divide by zero panic
    if n == 100 {
        let _ = 1 / 0; // This will panic
    }

    if n <= 1 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    for i in (3..=((n as f64).sqrt() as u32)).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}
