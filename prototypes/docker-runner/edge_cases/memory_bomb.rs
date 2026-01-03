/// Edge Case: Memory Bomb
/// This student code attempts to allocate excessive memory

pub fn fibonacci(n: u32) -> u64 {
    // Attempt to allocate a huge vector (should be killed by container memory limit)
    let mut _bomb: Vec<Vec<u8>> = Vec::new();
    for _ in 0..1000000 {
        _bomb.push(vec![0; 1024 * 1024]); // 1MB per allocation
    }

    // Actual implementation (unreachable)
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
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
