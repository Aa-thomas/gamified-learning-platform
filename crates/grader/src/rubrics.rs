//! Rubric loading and validation
//!
//! Loads JSON rubrics that define grading criteria for different artifact types.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::GraderError;

/// A grading rubric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    /// Type of artifact (e.g., "DESIGN", "README")
    pub artifact_type: String,
    /// Total possible points
    pub total_points: u32,
    /// Grading categories
    pub categories: Vec<RubricCategory>,
    /// Guidelines for letter grades
    #[serde(default)]
    pub grading_guidelines: GradingGuidelines,
    /// Required sections that must be present
    #[serde(default)]
    pub mandatory_sections: Vec<String>,
}

impl Rubric {
    /// Load a rubric from a JSON file
    pub fn from_file(path: &Path) -> Result<Self, GraderError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parse a rubric from JSON string
    pub fn from_json(json: &str) -> Result<Self, GraderError> {
        serde_json::from_str(json)
            .map_err(|e| GraderError::ParseError(format!("Failed to parse rubric: {}", e)))
    }

    /// Validate the rubric
    pub fn validate(&self) -> Result<(), GraderError> {
        // Check total points
        let sum: u32 = self.categories.iter().map(|c| c.points).sum();
        if sum != self.total_points {
            return Err(GraderError::ParseError(format!(
                "Category points sum ({}) doesn't match total_points ({})",
                sum, self.total_points
            )));
        }

        // Check each category has criteria
        for category in &self.categories {
            if category.criteria.is_empty() && category.indicators.is_none() {
                return Err(GraderError::ParseError(format!(
                    "Category '{}' has no criteria or indicators",
                    category.name
                )));
            }
        }

        Ok(())
    }

    /// Get the rubric as a formatted string for the LLM prompt
    pub fn to_prompt_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// A category within a rubric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCategory {
    /// Category name
    pub name: String,
    /// Points for this category
    pub points: u32,
    /// Detailed criteria (optional)
    #[serde(default)]
    pub criteria: Vec<Criterion>,
    /// Simple indicators (optional, alternative to criteria)
    #[serde(default)]
    pub indicators: Option<Indicators>,
}

/// A specific criterion within a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    /// Description of what's being evaluated
    pub description: String,
    /// Points for this criterion
    pub points: u32,
    /// Performance indicators
    pub indicators: Indicators,
}

/// Performance indicators for excellent/good/poor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicators {
    /// What excellent performance looks like
    pub excellent: String,
    /// What good performance looks like
    pub good: String,
    /// What poor performance looks like
    pub poor: String,
}

/// Grade range guidelines
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GradingGuidelines {
    #[serde(rename = "A (90-100)", default)]
    pub a_grade: String,
    #[serde(rename = "B (80-89)", default)]
    pub b_grade: String,
    #[serde(rename = "C (70-79)", default)]
    pub c_grade: String,
    #[serde(rename = "D (60-69)", default)]
    pub d_grade: String,
    #[serde(rename = "F (0-59)", default)]
    pub f_grade: String,
}

/// Built-in rubric definitions
pub struct BuiltInRubrics;

impl BuiltInRubrics {
    /// Get the DESIGN.md rubric
    pub fn design() -> Rubric {
        serde_json::from_str(DESIGN_RUBRIC_JSON).unwrap()
    }

    /// Get the README.md rubric
    pub fn readme() -> Rubric {
        serde_json::from_str(README_RUBRIC_JSON).unwrap()
    }

    /// Get rubric by artifact type
    pub fn get(artifact_type: &str) -> Option<Rubric> {
        match artifact_type.to_uppercase().as_str() {
            "DESIGN" | "DESIGN.MD" => Some(Self::design()),
            "README" | "README.MD" => Some(Self::readme()),
            _ => None,
        }
    }
}

const DESIGN_RUBRIC_JSON: &str = r#"{
    "artifact_type": "DESIGN.md",
    "total_points": 100,
    "categories": [
        {
            "name": "Architecture Overview",
            "points": 30,
            "criteria": [
                {
                    "description": "System components clearly identified",
                    "points": 15,
                    "indicators": {
                        "excellent": "All components named with clear responsibilities and boundaries",
                        "good": "Most components identified with basic descriptions",
                        "poor": "Missing components or unclear structure"
                    }
                },
                {
                    "description": "Component interactions documented",
                    "points": 15,
                    "indicators": {
                        "excellent": "Data flow and communication patterns clearly shown",
                        "good": "Basic interactions described",
                        "poor": "No interaction documentation"
                    }
                }
            ]
        },
        {
            "name": "Data Structures",
            "points": 25,
            "criteria": [
                {
                    "description": "Key data structures defined",
                    "points": 15,
                    "indicators": {
                        "excellent": "Structs/enums with fields, types, and constraints",
                        "good": "Basic structure definitions",
                        "poor": "Missing or incomplete definitions"
                    }
                },
                {
                    "description": "Data relationships explained",
                    "points": 10,
                    "indicators": {
                        "excellent": "Ownership, references, and relationships clear",
                        "good": "Some relationships mentioned",
                        "poor": "No relationship documentation"
                    }
                }
            ]
        },
        {
            "name": "API Design",
            "points": 25,
            "criteria": [
                {
                    "description": "Public interfaces documented",
                    "points": 15,
                    "indicators": {
                        "excellent": "Function signatures with parameters, return types, and error conditions",
                        "good": "Basic function descriptions",
                        "poor": "Missing API documentation"
                    }
                },
                {
                    "description": "Error handling strategy",
                    "points": 10,
                    "indicators": {
                        "excellent": "Error types defined with recovery strategies",
                        "good": "Basic error handling mentioned",
                        "poor": "No error handling discussion"
                    }
                }
            ]
        },
        {
            "name": "Technical Decisions",
            "points": 20,
            "criteria": [
                {
                    "description": "Design decisions justified",
                    "points": 10,
                    "indicators": {
                        "excellent": "Trade-offs explained with alternatives considered",
                        "good": "Basic rationale provided",
                        "poor": "No justification for decisions"
                    }
                },
                {
                    "description": "Future considerations",
                    "points": 10,
                    "indicators": {
                        "excellent": "Extensibility and scalability discussed",
                        "good": "Some future considerations",
                        "poor": "No forward thinking"
                    }
                }
            ]
        }
    ],
    "grading_guidelines": {
        "A (90-100)": "Comprehensive design covering all aspects with clear rationale and professional quality.",
        "B (80-89)": "Good design with minor gaps in documentation or rationale.",
        "C (70-79)": "Basic design present but missing important details.",
        "D (60-69)": "Incomplete design with significant gaps.",
        "F (0-59)": "Missing or severely lacking design documentation."
    },
    "mandatory_sections": [
        "Architecture overview",
        "Data structures",
        "Public API"
    ]
}"#;

const README_RUBRIC_JSON: &str = r#"{
    "artifact_type": "README.md",
    "total_points": 100,
    "categories": [
        {
            "name": "Project Overview",
            "points": 20,
            "criteria": [
                {
                    "description": "Clear project description",
                    "points": 10,
                    "indicators": {
                        "excellent": "Concise description explaining what the project does and why",
                        "good": "Basic description present",
                        "poor": "Missing or unclear description"
                    }
                },
                {
                    "description": "Key features listed",
                    "points": 10,
                    "indicators": {
                        "excellent": "Features clearly listed with brief explanations",
                        "good": "Some features mentioned",
                        "poor": "No feature list"
                    }
                }
            ]
        },
        {
            "name": "Installation",
            "points": 25,
            "criteria": [
                {
                    "description": "Prerequisites documented",
                    "points": 10,
                    "indicators": {
                        "excellent": "All dependencies with version requirements",
                        "good": "Basic prerequisites listed",
                        "poor": "Missing prerequisites"
                    }
                },
                {
                    "description": "Installation steps",
                    "points": 15,
                    "indicators": {
                        "excellent": "Step-by-step with copy-paste commands",
                        "good": "Basic installation instructions",
                        "poor": "Missing or incomplete instructions"
                    }
                }
            ]
        },
        {
            "name": "Usage",
            "points": 30,
            "criteria": [
                {
                    "description": "Basic usage examples",
                    "points": 15,
                    "indicators": {
                        "excellent": "Multiple examples with explanations",
                        "good": "At least one working example",
                        "poor": "No usage examples"
                    }
                },
                {
                    "description": "Command reference",
                    "points": 15,
                    "indicators": {
                        "excellent": "All commands/options documented",
                        "good": "Main commands documented",
                        "poor": "Missing command documentation"
                    }
                }
            ]
        },
        {
            "name": "Additional Sections",
            "points": 25,
            "criteria": [
                {
                    "description": "Contributing guidelines",
                    "points": 10,
                    "indicators": {
                        "excellent": "Clear contribution process and code standards",
                        "good": "Basic contributing info",
                        "poor": "No contributing section"
                    }
                },
                {
                    "description": "License and credits",
                    "points": 5,
                    "indicators": {
                        "excellent": "License clearly stated with acknowledgments",
                        "good": "License mentioned",
                        "poor": "No license information"
                    }
                },
                {
                    "description": "Testing information",
                    "points": 10,
                    "indicators": {
                        "excellent": "How to run tests with coverage info",
                        "good": "Basic test instructions",
                        "poor": "No testing documentation"
                    }
                }
            ]
        }
    ],
    "grading_guidelines": {
        "A (90-100)": "Complete README with all sections, clear examples, and professional presentation.",
        "B (80-89)": "Good README with minor gaps in documentation.",
        "C (70-79)": "Basic README covering essentials but missing details.",
        "D (60-69)": "Incomplete README with significant gaps.",
        "F (0-59)": "Missing or severely lacking documentation."
    },
    "mandatory_sections": [
        "Project description",
        "Installation instructions",
        "Usage examples"
    ]
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_design_rubric() {
        let rubric = BuiltInRubrics::design();
        assert_eq!(rubric.artifact_type, "DESIGN.md");
        assert_eq!(rubric.total_points, 100);
        assert_eq!(rubric.categories.len(), 4);
    }

    #[test]
    fn test_parse_readme_rubric() {
        let rubric = BuiltInRubrics::readme();
        assert_eq!(rubric.artifact_type, "README.md");
        assert_eq!(rubric.total_points, 100);
    }

    #[test]
    fn test_validate_rubric() {
        let rubric = BuiltInRubrics::design();
        assert!(rubric.validate().is_ok());
    }

    #[test]
    fn test_invalid_rubric_points() {
        let json = r#"{
            "artifact_type": "TEST",
            "total_points": 100,
            "categories": [
                {
                    "name": "Test",
                    "points": 50,
                    "criteria": [{"description": "x", "points": 50, "indicators": {"excellent": "a", "good": "b", "poor": "c"}}]
                }
            ]
        }"#;

        let rubric = Rubric::from_json(json).unwrap();
        let result = rubric.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("doesn't match"));
    }

    #[test]
    fn test_get_by_type() {
        assert!(BuiltInRubrics::get("DESIGN").is_some());
        assert!(BuiltInRubrics::get("design.md").is_some());
        assert!(BuiltInRubrics::get("README").is_some());
        assert!(BuiltInRubrics::get("unknown").is_none());
    }

    #[test]
    fn test_category_points_sum() {
        let rubric = BuiltInRubrics::design();
        let sum: u32 = rubric.categories.iter().map(|c| c.points).sum();
        assert_eq!(sum, 100);
    }

    #[test]
    fn test_to_prompt_string() {
        let rubric = BuiltInRubrics::design();
        let prompt = rubric.to_prompt_string();
        assert!(prompt.contains("DESIGN.md"));
        assert!(prompt.contains("Architecture"));
    }

    #[test]
    fn test_mandatory_sections() {
        let rubric = BuiltInRubrics::design();
        assert!(!rubric.mandatory_sections.is_empty());
        assert!(rubric.mandatory_sections.iter().any(|s| s.contains("Architecture")));
    }
}
