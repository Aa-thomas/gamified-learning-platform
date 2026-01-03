/// Test script for Docker Runner
/// This demonstrates how to use the runner with different edge cases

use std::fs;
use std::path::Path;

mod runner;
use runner::{DockerRunner, VerificationResult};

fn main() {
    println!("=== Docker Runner Edge Case Testing ===\n");

    // Check if Docker is available
    match DockerRunner::check_docker_available() {
        Ok(true) => println!("✓ Docker is available\n"),
        Ok(false) => {
            println!("✗ Docker is installed but not working properly\n");
            return;
        }
        Err(e) => {
            println!("✗ {}\n", e);
            println!("This test requires Docker to be installed.");
            println!("Please install Docker and try again.\n");
            return;
        }
    }

    let runner = DockerRunner::new();
    let challenge_dir = Path::new("sample_challenge");
    let edge_cases_dir = Path::new("edge_cases");

    // Build Docker image
    println!("Building Docker image...");
    match runner.build_image(Path::new(".")) {
        Ok(_) => println!("✓ Docker image built successfully\n"),
        Err(e) => {
            println!("✗ Failed to build Docker image: {}\n", e);
            return;
        }
    }

    // Test 1: Correct Solution
    println!("--- Test 1: Correct Solution ---");
    test_edge_case(
        &runner,
        challenge_dir,
        edge_cases_dir.join("correct_solution.rs"),
        "Should pass all tests",
    );

    // Test 2: Infinite Loop
    println!("\n--- Test 2: Infinite Loop (Timeout) ---");
    test_edge_case(
        &runner,
        challenge_dir,
        edge_cases_dir.join("infinite_loop.rs"),
        "Should timeout after 30 seconds",
    );

    // Test 3: Compile Error
    println!("\n--- Test 3: Compile Error ---");
    test_edge_case(
        &runner,
        challenge_dir,
        edge_cases_dir.join("compile_error.rs"),
        "Should capture compile errors",
    );

    // Test 4: Panic in Test
    println!("\n--- Test 4: Panic in Test ---");
    test_edge_case(
        &runner,
        challenge_dir,
        edge_cases_dir.join("panic_test.rs"),
        "Should report test failures from panics",
    );

    // Test 5: Memory Bomb
    println!("\n--- Test 5: Memory Bomb ---");
    test_edge_case(
        &runner,
        challenge_dir,
        edge_cases_dir.join("memory_bomb.rs"),
        "Should be killed by container memory limit",
    );

    // Test 6: Container Cleanup
    println!("\n--- Test 6: Container Cleanup ---");
    println!("Checking for orphaned containers...");
    if check_orphaned_containers() {
        println!("✓ No orphaned containers found");
    } else {
        println!("✗ Orphaned containers detected!");
    }

    println!("\n=== All Tests Complete ===");
}

fn test_edge_case(runner: &DockerRunner, challenge_dir: &Path, code_file: Path, description: &str) {
    println!("Description: {}", description);

    let student_code = match fs::read_to_string(&code_file) {
        Ok(code) => code,
        Err(e) => {
            println!("✗ Failed to read code file: {}", e);
            return;
        }
    };

    match runner.run_verification(challenge_dir, &student_code) {
        Ok(result) => print_result(&result),
        Err(e) => println!("✗ Error running verification: {}", e),
    }
}

fn print_result(result: &VerificationResult) {
    println!("Success: {}", result.success);
    println!("Duration: {:.2}s", result.duration.as_secs_f64());
    println!("Tests Passed: {}", result.tests_passed);
    println!("Tests Failed: {}", result.tests_failed);
    println!("Compile Error: {}", result.compile_error);
    println!("Timeout: {}", result.timeout);

    if !result.stdout.is_empty() {
        println!("\nStdout (first 500 chars):");
        println!("{}", &result.stdout[..result.stdout.len().min(500)]);
    }

    if !result.stderr.is_empty() {
        println!("\nStderr (first 500 chars):");
        println!("{}", &result.stderr[..result.stderr.len().min(500)]);
    }
}

fn check_orphaned_containers() -> bool {
    use std::process::Command;

    let output = Command::new("docker")
        .args(&["ps", "-a", "--filter", "name=challenge-", "--format", "{{.Names}}"])
        .output()
        .expect("Failed to check containers");

    let containers = String::from_utf8_lossy(&output.stdout);
    containers.trim().is_empty()
}
