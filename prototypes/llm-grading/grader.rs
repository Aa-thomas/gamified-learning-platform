/// LLM-based artifact grader
///
/// This module provides functionality to grade student artifacts (DESIGN.md, README.md, etc.)
/// using OpenAI's GPT-4 API with structured rubrics.

use std::fs;
use std::path::Path;
use std::time::Instant;

/// Represents a grading rubric loaded from JSON
#[derive(Debug, Clone)]
pub struct Rubric {
    pub artifact_type: String,
    pub total_points: u32,
    pub rubric_json: String, // Full JSON for prompt injection
}

impl Rubric {
    /// Load a rubric from a JSON file
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read rubric file: {}", e))?;

        // Parse to extract artifact_type and total_points
        // In a real implementation, use serde_json
        let artifact_type = extract_json_field(&content, "artifact_type")
            .unwrap_or_else(|| "unknown".to_string());
        let total_points = extract_json_field(&content, "total_points")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Ok(Self {
            artifact_type,
            total_points,
            rubric_json: content,
        })
    }
}

/// Result of grading an artifact
#[derive(Debug, Clone)]
pub struct GradeResult {
    pub score: u32,
    pub max_score: u32,
    pub percentage: f64,
    pub reasoning: String,
    pub category_scores: Vec<CategoryScore>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CategoryScore {
    pub category: String,
    pub score: u32,
    pub max_score: u32,
    pub feedback: String,
}

/// LLM grader that uses OpenAI API
pub struct LLMGrader {
    api_key: String,
    model: String,
    temperature: f64,
}

impl LLMGrader {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "gpt-4".to_string(),
            temperature: 0.3, // Low temperature for consistency
        }
    }

    /// Grade an artifact using the provided rubric
    pub fn grade(
        &self,
        artifact_content: &str,
        rubric: &Rubric,
    ) -> Result<GradeResult, String> {
        let start = Instant::now();

        // Build the grading prompt
        let prompt = self.build_grading_prompt(artifact_content, rubric);

        // Call OpenAI API
        let response = self.call_openai_api(&prompt)?;

        // Parse the response
        let grade_result = self.parse_grading_response(&response, rubric, start.elapsed().as_millis() as u64)?;

        Ok(grade_result)
    }

    /// Build the prompt for GPT-4
    fn build_grading_prompt(&self, artifact: &str, rubric: &Rubric) -> String {
        format!(
            r#"You are an expert code reviewer grading a student's {} artifact.

# GRADING RUBRIC

{}

# STUDENT ARTIFACT

```
{}
```

# INSTRUCTIONS

1. Carefully read the student's artifact
2. Evaluate it against each category in the rubric
3. Provide a score for each category (be strict but fair)
4. Write specific feedback explaining the score
5. Calculate the total score

# OUTPUT FORMAT

Respond in this exact JSON format:

```json
{{
  "total_score": <number>,
  "max_score": {},
  "overall_feedback": "<2-3 sentences summarizing the artifact quality>",
  "category_scores": [
    {{
      "category": "<category name>",
      "score": <number>,
      "max_score": <number>,
      "feedback": "<specific feedback for this category>"
    }}
  ]
}}
```

Be specific in your feedback. Point out what was done well and what was missing or could be improved.
"#,
            rubric.artifact_type,
            rubric.rubric_json,
            artifact,
            rubric.total_points
        )
    }

    /// Call OpenAI API (placeholder - requires actual HTTP client)
    fn call_openai_api(&self, prompt: &str) -> Result<String, String> {
        // In a real implementation, this would make an HTTP POST to:
        // https://api.openai.com/v1/chat/completions
        //
        // Example using reqwest:
        // ```
        // let client = reqwest::blocking::Client::new();
        // let response = client.post("https://api.openai.com/v1/chat/completions")
        //     .header("Authorization", format!("Bearer {}", self.api_key))
        //     .json(&json!({
        //         "model": self.model,
        //         "temperature": self.temperature,
        //         "messages": [{
        //             "role": "user",
        //             "content": prompt
        //         }]
        //     }))
        //     .send()?;
        // ```

        // For prototype purposes, return a mock response
        Err("OpenAI API not configured. This is a prototype demonstrating the grading flow.".to_string())
    }

    /// Parse GPT-4's response into a GradeResult
    fn parse_grading_response(
        &self,
        response: &str,
        rubric: &Rubric,
        latency_ms: u64,
    ) -> Result<GradeResult, String> {
        // In a real implementation, parse the JSON response
        // For prototype, return a mock result
        Ok(GradeResult {
            score: 0,
            max_score: rubric.total_points,
            percentage: 0.0,
            reasoning: "Mock result - API not configured".to_string(),
            category_scores: vec![],
            latency_ms,
        })
    }

    /// Grade an artifact multiple times to test consistency
    pub fn grade_multiple(
        &self,
        artifact_content: &str,
        rubric: &Rubric,
        iterations: usize,
    ) -> Result<Vec<GradeResult>, String> {
        let mut results = Vec::new();

        for i in 0..iterations {
            println!("Grading iteration {}/{}...", i + 1, iterations);
            let result = self.grade(artifact_content, rubric)?;
            results.push(result);
        }

        Ok(results)
    }
}

/// Calculate consistency metrics from multiple grading runs
pub struct ConsistencyMetrics {
    pub mean_score: f64,
    pub std_deviation: f64,
    pub min_score: u32,
    pub max_score: u32,
    pub variance: f64,
}

impl ConsistencyMetrics {
    pub fn from_results(results: &[GradeResult]) -> Self {
        if results.is_empty() {
            return Self {
                mean_score: 0.0,
                std_deviation: 0.0,
                min_score: 0,
                max_score: 0,
                variance: 0.0,
            };
        }

        let scores: Vec<f64> = results.iter().map(|r| r.score as f64).collect();
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;

        let variance = scores
            .iter()
            .map(|score| {
                let diff = score - mean;
                diff * diff
            })
            .sum::<f64>()
            / scores.len() as f64;

        let std_dev = variance.sqrt();

        Self {
            mean_score: mean,
            std_deviation: std_dev,
            min_score: results.iter().map(|r| r.score).min().unwrap(),
            max_score: results.iter().map(|r| r.score).max().unwrap(),
            variance,
        }
    }

    /// Check if consistency is acceptable (within ±5 points)
    pub fn is_consistent(&self) -> bool {
        self.std_deviation <= 5.0
    }
}

// Helper function to extract JSON field (simplified)
fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", field);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = json[value_start..].find('"') {
            return Some(json[value_start..value_start + end].to_string());
        }
    }

    // Try numeric field
    let pattern = format!("\"{}\": ", field);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        let rest = &json[value_start..];
        let end = rest
            .find(|c: char| c == ',' || c == '}' || c == '\n')
            .unwrap_or(rest.len());
        return Some(rest[..end].trim().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_metrics() {
        let results = vec![
            GradeResult {
                score: 85,
                max_score: 100,
                percentage: 85.0,
                reasoning: "Test".to_string(),
                category_scores: vec![],
                latency_ms: 1000,
            },
            GradeResult {
                score: 87,
                max_score: 100,
                percentage: 87.0,
                reasoning: "Test".to_string(),
                category_scores: vec![],
                latency_ms: 1000,
            },
            GradeResult {
                score: 84,
                max_score: 100,
                percentage: 84.0,
                reasoning: "Test".to_string(),
                category_scores: vec![],
                latency_ms: 1000,
            },
        ];

        let metrics = ConsistencyMetrics::from_results(&results);
        assert!(metrics.is_consistent()); // Should be within ±5 points
        assert_eq!(metrics.min_score, 84);
        assert_eq!(metrics.max_score, 87);
    }

    #[test]
    fn test_rubric_loading() {
        // Test would require actual rubric file
        // This is a placeholder
        assert!(true);
    }
}

// Example usage
#[allow(dead_code)]
fn example_usage() {
    // Load rubric
    let rubric = Rubric::from_file(Path::new("rubrics/design_rubric.json"))
        .expect("Failed to load rubric");

    // Load artifact
    let artifact = fs::read_to_string("sample_artifacts/design_good.md")
        .expect("Failed to load artifact");

    // Create grader (requires OpenAI API key)
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not set");
    let grader = LLMGrader::new(api_key);

    // Grade multiple times to test consistency
    match grader.grade_multiple(&artifact, &rubric, 5) {
        Ok(results) => {
            let metrics = ConsistencyMetrics::from_results(&results);

            println!("Grading Results:");
            println!("  Mean Score: {:.2}", metrics.mean_score);
            println!("  Std Dev: {:.2}", metrics.std_deviation);
            println!("  Range: {}-{}", metrics.min_score, metrics.max_score);
            println!("  Consistent: {}", metrics.is_consistent());

            for (i, result) in results.iter().enumerate() {
                println!("\nRun {}: {}/{} ({:.1}%)",
                    i + 1,
                    result.score,
                    result.max_score,
                    result.percentage
                );
                println!("  Reasoning: {}", result.reasoning);
                println!("  Latency: {}ms", result.latency_ms);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
