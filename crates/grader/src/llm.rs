//! LLM-based artifact grading using OpenAI
//!
//! Provides grading functionality using GPT-4 with retry logic and caching.

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use std::time::Instant;

use crate::cache::GradeCache;
use crate::error::GraderError;
use crate::rubrics::Rubric;
use crate::types::{CategoryScore, GradeResult, GraderConfig};

/// LLM-based grader using OpenAI
pub struct LLMGrader {
    client: Client<OpenAIConfig>,
    config: GraderConfig,
}

impl LLMGrader {
    /// Create a new LLM grader with the given API key
    pub fn new(api_key: &str) -> Self {
        let openai_config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(openai_config);
        
        Self {
            client,
            config: GraderConfig::default(),
        }
    }

    /// Create a new LLM grader with custom configuration
    pub fn with_config(api_key: &str, config: GraderConfig) -> Self {
        let openai_config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(openai_config);
        
        Self { client, config }
    }

    /// Grade an artifact using the provided rubric
    pub async fn grade(
        &self,
        artifact_content: &str,
        rubric: &Rubric,
    ) -> Result<GradeResult, GraderError> {
        let start = Instant::now();

        // Build the prompt
        let system_message = self.build_system_message();
        let user_message = self.build_user_message(artifact_content, rubric);

        // Make the API call
        let response = self.call_api(&system_message, &user_message).await?;

        // Parse the response
        let latency_ms = start.elapsed().as_millis() as u64;
        self.parse_response(&response, latency_ms)
    }

    /// Grade an artifact with caching
    pub async fn grade_with_cache(
        &self,
        artifact_content: &str,
        rubric: &Rubric,
        cache: &GradeCache,
    ) -> Result<GradeResult, GraderError> {
        // Check cache first
        if let Some(cached) = cache.get(artifact_content, &rubric.artifact_type)? {
            return Ok(cached);
        }

        // Cache miss, call LLM
        let result = self.grade(artifact_content, rubric).await?;

        // Store in cache
        cache.set(artifact_content, &rubric.artifact_type, &result)?;

        Ok(result)
    }

    /// Build the system message for the LLM
    fn build_system_message(&self) -> String {
        r#"You are an expert code reviewer and educator grading student project artifacts for a Rust bootcamp.

Your role is to:
1. Evaluate artifacts against structured rubrics
2. Provide constructive, specific feedback
3. Be strict but fair in scoring
4. Help students improve their technical writing

Grading philosophy:
- Reward clarity, completeness, and technical depth
- Penalize vagueness, missing sections, and superficial analysis
- Focus on substance over style (but clarity matters)
- Compare to professional-level documentation"#
            .to_string()
    }

    /// Build the user message with artifact and rubric
    fn build_user_message(&self, artifact: &str, rubric: &Rubric) -> String {
        format!(
            r#"# GRADING TASK

## Artifact Type: {}

## Rubric
{}

## Student Submission
```
{}
```

## Instructions
1. Read the student's artifact carefully
2. Evaluate against each category in the rubric
3. Score each criterion using the indicators (excellent/good/poor)
4. Provide specific feedback citing examples from the artifact
5. Calculate total score

## Output Format
Respond with ONLY valid JSON in this exact format (no markdown, no code blocks):

{{
  "total_score": <number 0-100>,
  "overall_feedback": "<2-3 sentences summarizing quality and areas for improvement>",
  "category_scores": [
    {{
      "category": "<category name>",
      "score": <number>,
      "max_score": <number>,
      "feedback": "<specific feedback with examples>"
    }}
  ]
}}

Be specific in your feedback. Quote or reference specific parts of the artifact."#,
            rubric.artifact_type,
            rubric.to_prompt_string(),
            artifact
        )
    }

    /// Call the OpenAI API
    async fn call_api(
        &self,
        system_message: &str,
        user_message: &str,
    ) -> Result<String, GraderError> {
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_message)
                    .build()
                    .map_err(|e| GraderError::ApiError(e.to_string()))?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_message)
                    .build()
                    .map_err(|e| GraderError::ApiError(e.to_string()))?,
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens)
            .messages(messages)
            .build()
            .map_err(|e| GraderError::ApiError(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| GraderError::ParseError("Empty response from LLM".to_string()))?;

        Ok(content)
    }

    /// Parse the LLM response into a GradeResult
    fn parse_response(&self, response: &str, latency_ms: u64) -> Result<GradeResult, GraderError> {
        // Try to extract JSON from the response (in case there's extra text)
        let json_str = extract_json(response)?;

        let parsed: LLMResponse = serde_json::from_str(&json_str)
            .map_err(|e| GraderError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        let category_scores: Vec<CategoryScore> = parsed
            .category_scores
            .into_iter()
            .map(|c| CategoryScore {
                category: c.category,
                score: c.score,
                max_score: c.max_score,
                feedback: c.feedback,
            })
            .collect();

        Ok(GradeResult {
            score: parsed.total_score,
            max_score: 100,
            overall_feedback: parsed.overall_feedback,
            category_scores,
            from_cache: false,
            latency_ms,
        })
    }
}

/// Extract JSON from a potentially wrapped response
fn extract_json(response: &str) -> Result<String, GraderError> {
    let trimmed = response.trim();

    // If it starts with {, assume it's pure JSON
    if trimmed.starts_with('{') {
        return Ok(trimmed.to_string());
    }

    // Try to find JSON in code blocks
    if let Some(start) = trimmed.find("```json") {
        let after_marker = &trimmed[start + 7..];
        if let Some(end) = after_marker.find("```") {
            return Ok(after_marker[..end].trim().to_string());
        }
    }

    // Try to find JSON block without language marker
    if let Some(start) = trimmed.find("```") {
        let after_marker = &trimmed[start + 3..];
        if let Some(end) = after_marker.find("```") {
            let content = after_marker[..end].trim();
            if content.starts_with('{') {
                return Ok(content.to_string());
            }
        }
    }

    // Try to find { ... } anywhere
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return Ok(trimmed[start..=end].to_string());
            }
        }
    }

    Err(GraderError::ParseError(
        "Could not find valid JSON in response".to_string(),
    ))
}

/// Expected LLM response structure
#[derive(serde::Deserialize)]
struct LLMResponse {
    total_score: u32,
    overall_feedback: String,
    category_scores: Vec<LLMCategoryScore>,
}

#[derive(serde::Deserialize)]
struct LLMCategoryScore {
    category: String,
    score: u32,
    max_score: u32,
    feedback: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_pure() {
        let response = r#"{"total_score": 85, "overall_feedback": "Good", "category_scores": []}"#;
        let json = extract_json(response).unwrap();
        assert!(json.starts_with('{'));
    }

    #[test]
    fn test_extract_json_with_code_block() {
        let response = r#"Here is the grade:
```json
{"total_score": 85, "overall_feedback": "Good", "category_scores": []}
```"#;
        let json = extract_json(response).unwrap();
        assert!(json.contains("total_score"));
    }

    #[test]
    fn test_extract_json_with_surrounding_text() {
        let response = r#"I've evaluated the artifact.
{"total_score": 85, "overall_feedback": "Good", "category_scores": []}
That's my assessment."#;
        let json = extract_json(response).unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_parse_response() {
        let grader = LLMGrader::new("test-key");
        let response = r#"{
            "total_score": 85,
            "overall_feedback": "Good work overall!",
            "category_scores": [
                {
                    "category": "Architecture",
                    "score": 25,
                    "max_score": 30,
                    "feedback": "Clear structure"
                }
            ]
        }"#;

        let result = grader.parse_response(response, 500).unwrap();
        assert_eq!(result.score, 85);
        assert_eq!(result.overall_feedback, "Good work overall!");
        assert_eq!(result.category_scores.len(), 1);
        assert!(!result.from_cache);
    }

    #[test]
    fn test_build_system_message() {
        let grader = LLMGrader::new("test-key");
        let msg = grader.build_system_message();
        assert!(msg.contains("expert code reviewer"));
        assert!(msg.contains("Rust bootcamp"));
    }

    #[test]
    fn test_build_user_message() {
        let grader = LLMGrader::new("test-key");
        let rubric = crate::rubrics::BuiltInRubrics::design();
        let msg = grader.build_user_message("# Test Artifact", &rubric);
        
        assert!(msg.contains("DESIGN.md"));
        assert!(msg.contains("# Test Artifact"));
        assert!(msg.contains("total_score"));
    }

    #[test]
    fn test_extract_json_fails_on_invalid() {
        let response = "This has no JSON at all";
        let result = extract_json(response);
        assert!(result.is_err());
    }
}
