//! Content validator module
//!
//! Validates manifest.json structure, file paths, and content schemas.

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub weeks: Vec<Week>,
    pub checkpoints: Vec<Checkpoint>,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Deserialize)]
pub struct Week {
    pub id: String,
    pub title: String,
    pub description: String,
    pub days: Vec<Day>,
}

#[derive(Debug, Deserialize)]
pub struct Day {
    pub id: String,
    pub title: String,
    pub description: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub estimated_minutes: u32,
    pub xp_reward: u32,
    pub content_path: String,
    pub skills: Vec<String>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub title: String,
    pub week: String,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    #[serde(rename = "type")]
    pub question_type: String,
    pub options: Vec<String>,
    #[serde(default)]
    pub correct_answer: Option<usize>,
    #[serde(default)]
    pub correct_answers: Option<Vec<usize>>,
    pub explanation: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub starter_code: String,
    pub test_code: String,
    pub solution: String,
    pub hints: Vec<String>,
    pub difficulty: String,
    pub skills: Vec<String>,
}

pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.errors.is_empty() {
            writeln!(f, "\n{}", "Errors:".red().bold())?;
            for err in &self.errors {
                writeln!(f, "  {} {}", "✗".red(), err)?;
            }
        }
        
        if !self.warnings.is_empty() {
            writeln!(f, "\n{}", "Warnings:".yellow().bold())?;
            for warn in &self.warnings {
                writeln!(f, "  {} {}", "⚠".yellow(), warn)?;
            }
        }
        
        if !self.info.is_empty() {
            writeln!(f, "\n{}", "Info:".blue().bold())?;
            for info in &self.info {
                writeln!(f, "  {} {}", "ℹ".blue(), info)?;
            }
        }
        
        if self.errors.is_empty() {
            writeln!(f, "\n{}", "✓ All validations passed!".green().bold())?;
        } else {
            writeln!(f, "\n{}", format!("✗ {} error(s) found", self.errors.len()).red().bold())?;
        }
        
        Ok(())
    }
}

pub fn validate_content(content_path: &Path) -> Result<ValidationReport> {
    let mut report = ValidationReport {
        errors: Vec::new(),
        warnings: Vec::new(),
        info: Vec::new(),
    };
    
    // Load manifest
    let manifest_path = content_path.join("manifest.json");
    if !manifest_path.exists() {
        report.errors.push("manifest.json not found".to_string());
        return Ok(report);
    }
    
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .context("Failed to read manifest.json")?;
    
    let manifest: Manifest = serde_json::from_str(&manifest_content)
        .context("Failed to parse manifest.json")?;
    
    report.info.push(format!("Found manifest: {}", manifest.title));
    
    // Collect all defined skill IDs
    let skill_ids: HashSet<&str> = manifest.skills.iter().map(|s| s.id.as_str()).collect();
    
    // Collect all node IDs for prerequisite validation
    let mut node_ids: HashSet<String> = HashSet::new();
    
    // Validate weeks and nodes
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                // Check for duplicate node IDs
                if !node_ids.insert(node.id.clone()) {
                    report.errors.push(format!("Duplicate node ID: {}", node.id));
                }
                
                // Check content file exists
                let content_file = content_path.join(&node.content_path);
                if !content_file.exists() {
                    report.errors.push(format!(
                        "Missing content file for '{}': {}",
                        node.id, node.content_path
                    ));
                } else {
                    // Validate content file based on type
                    if let Err(e) = validate_content_file(&content_file, &node.node_type) {
                        report.errors.push(format!(
                            "Invalid content file '{}': {}",
                            node.content_path, e
                        ));
                    }
                }
                
                // Check skills are defined
                for skill in &node.skills {
                    if !skill_ids.contains(skill.as_str()) {
                        report.warnings.push(format!(
                            "Node '{}' references undefined skill: {}",
                            node.id, skill
                        ));
                    }
                }
                
                // Validate difficulty
                if !["easy", "medium", "hard", "very_hard"].contains(&node.difficulty.as_str()) {
                    report.warnings.push(format!(
                        "Node '{}' has non-standard difficulty: {}",
                        node.id, node.difficulty
                    ));
                }
                
                // Validate node type
                if !["lecture", "quiz", "mini-challenge", "checkpoint"].contains(&node.node_type.as_str()) {
                    report.warnings.push(format!(
                        "Node '{}' has unknown type: {}",
                        node.id, node.node_type
                    ));
                }
            }
        }
    }
    
    // Validate prerequisites (second pass)
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                for prereq in &node.prerequisites {
                    if !node_ids.contains(prereq) {
                        report.errors.push(format!(
                            "Node '{}' has invalid prerequisite: {}",
                            node.id, prereq
                        ));
                    }
                }
            }
        }
    }
    
    Ok(report)
}

fn validate_content_file(path: &Path, node_type: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    
    match node_type {
        "lecture" => {
            // Just check it's non-empty markdown
            if content.trim().is_empty() {
                anyhow::bail!("Lecture file is empty");
            }
            if !content.starts_with('#') {
                anyhow::bail!("Lecture should start with a heading");
            }
        }
        "quiz" => {
            let quiz: Quiz = serde_json::from_str(&content)
                .context("Invalid quiz JSON")?;
            if quiz.questions.is_empty() {
                anyhow::bail!("Quiz has no questions");
            }
            for q in &quiz.questions {
                if q.options.len() < 2 {
                    anyhow::bail!("Question '{}' needs at least 2 options", q.id);
                }
                // Validate correct answer is within bounds
                if let Some(idx) = q.correct_answer {
                    if idx >= q.options.len() {
                        anyhow::bail!("Question '{}' correct_answer index out of bounds", q.id);
                    }
                }
            }
        }
        "mini-challenge" => {
            let challenge: Challenge = serde_json::from_str(&content)
                .context("Invalid challenge JSON")?;
            if challenge.starter_code.is_empty() {
                anyhow::bail!("Challenge has no starter code");
            }
            if challenge.test_code.is_empty() {
                anyhow::bail!("Challenge has no test code");
            }
        }
        _ => {}
    }
    
    Ok(())
}

pub fn content_stats(content_path: &Path) -> Result<String> {
    let manifest_path = content_path.join("manifest.json");
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .context("Failed to read manifest.json")?;
    
    let manifest: Manifest = serde_json::from_str(&manifest_content)
        .context("Failed to parse manifest.json")?;
    
    let mut total_nodes = 0;
    let mut lectures = 0;
    let mut quizzes = 0;
    let mut challenges = 0;
    let mut total_xp = 0;
    let mut total_minutes = 0;
    
    for week in &manifest.weeks {
        for day in &week.days {
            for node in &day.nodes {
                total_nodes += 1;
                total_xp += node.xp_reward;
                total_minutes += node.estimated_minutes;
                
                match node.node_type.as_str() {
                    "lecture" => lectures += 1,
                    "quiz" => quizzes += 1,
                    "mini-challenge" => challenges += 1,
                    _ => {}
                }
            }
        }
    }
    
    let stats = format!(
        r#"
{}
  Weeks: {}
  Total nodes: {}
    - Lectures: {}
    - Quizzes: {}
    - Challenges: {}
  
{}
  Total XP available: {}
  Total time: {} minutes ({:.1} hours)
  
{}
  Skills defined: {}
  Checkpoints: {}
"#,
        "Content Overview".cyan().bold(),
        manifest.weeks.len(),
        total_nodes,
        lectures,
        quizzes,
        challenges,
        "Progression".cyan().bold(),
        total_xp,
        total_minutes,
        total_minutes as f64 / 60.0,
        "Skills & Checkpoints".cyan().bold(),
        manifest.skills.len(),
        manifest.checkpoints.len(),
    );
    
    Ok(stats)
}
