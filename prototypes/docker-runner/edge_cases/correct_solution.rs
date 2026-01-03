/// Edge Case: Correct Solution
/// This is the correct implementation that should pass all tests

pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
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
