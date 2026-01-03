/// Edge Case: Infinite Loop
/// This student code contains an infinite loop that should be killed by the timeout

pub fn fibonacci(n: u32) -> u64 {
    // Infinite loop - should trigger timeout
    loop {
        // Keep computing to consume CPU
        let _ = (0..1000000).sum::<u64>();
    }
}

pub fn is_prime(n: u32) -> bool {
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
