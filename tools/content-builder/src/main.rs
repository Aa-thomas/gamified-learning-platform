//! Content Builder CLI
//!
//! Tool for building, validating, and analyzing course content.

mod validator;

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "content-builder")]
#[command(about = "Build and validate course content", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate content manifest and all referenced files
    Validate {
        /// Path to content directory (default: ./content)
        #[arg(short, long, default_value = "./content")]
        path: PathBuf,
    },
    /// Show content statistics
    Stats {
        /// Path to content directory (default: ./content)
        #[arg(short, long, default_value = "./content")]
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path } => {
            println!("{}", "Validating content...".cyan().bold());
            match validator::validate_content(&path) {
                Ok(report) => {
                    println!("\n{}", "Validation Results:".green().bold());
                    println!("{}", report);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Stats { path } => {
            println!("{}", "Content Statistics:".cyan().bold());
            match validator::content_stats(&path) {
                Ok(stats) => println!("{}", stats),
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}
