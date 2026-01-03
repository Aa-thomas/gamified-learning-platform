use sample_challenge::{fibonacci, is_prime};

#[test]
fn test_fibonacci_base_cases() {
    assert_eq!(fibonacci(0), 0);
    assert_eq!(fibonacci(1), 1);
}

#[test]
fn test_fibonacci_small_numbers() {
    assert_eq!(fibonacci(2), 1);
    assert_eq!(fibonacci(3), 2);
    assert_eq!(fibonacci(4), 3);
    assert_eq!(fibonacci(5), 5);
}

#[test]
fn test_fibonacci_larger_numbers() {
    assert_eq!(fibonacci(10), 55);
    assert_eq!(fibonacci(15), 610);
    assert_eq!(fibonacci(20), 6765);
}

#[test]
fn test_is_prime_small_primes() {
    assert_eq!(is_prime(2), true);
    assert_eq!(is_prime(3), true);
    assert_eq!(is_prime(5), true);
    assert_eq!(is_prime(7), true);
}

#[test]
fn test_is_prime_non_primes() {
    assert_eq!(is_prime(0), false);
    assert_eq!(is_prime(1), false);
    assert_eq!(is_prime(4), false);
    assert_eq!(is_prime(6), false);
    assert_eq!(is_prime(8), false);
    assert_eq!(is_prime(9), false);
}

#[test]
fn test_is_prime_larger_numbers() {
    assert_eq!(is_prime(17), true);
    assert_eq!(is_prime(97), true);
    assert_eq!(is_prime(100), false);
    assert_eq!(is_prime(121), false);
}
