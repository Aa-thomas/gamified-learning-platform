use crate::models::quiz::Quiz;
use std::collections::HashMap;

/// Grade a quiz and return (score, correct_count, total_questions)
pub fn grade_quiz(quiz: &Quiz, answers: &HashMap<String, String>) -> (i32, usize, usize) {
    let mut score = 0;
    let mut correct_count = 0;
    let total = quiz.questions.len();

    for question in &quiz.questions {
        let user_answer = answers.get(&question.id);
        let is_correct = user_answer
            .map(|ans| ans == &question.correct_answer)
            .unwrap_or(false);

        if is_correct {
            score += question.points;
            correct_count += 1;
        }
    }

    (score, correct_count, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::quiz::{Question, QuestionOption};

    fn create_test_quiz() -> Quiz {
        Quiz {
            id: "test-quiz".to_string(),
            title: "Test Quiz".to_string(),
            description: "A test quiz".to_string(),
            difficulty: "Easy".to_string(),
            skills: vec!["rust".to_string()],
            passing_score: 70,
            time_limit_seconds: None,
            questions: vec![
                Question {
                    id: "q1".to_string(),
                    question_type: "multiple_choice".to_string(),
                    prompt: "What is 2+2?".to_string(),
                    code_snippet: None,
                    options: vec![
                        QuestionOption {
                            id: "a".to_string(),
                            text: "3".to_string(),
                        },
                        QuestionOption {
                            id: "b".to_string(),
                            text: "4".to_string(),
                        },
                        QuestionOption {
                            id: "c".to_string(),
                            text: "5".to_string(),
                        },
                    ],
                    correct_answer: "b".to_string(),
                    explanation: "2+2=4".to_string(),
                    points: 10,
                },
                Question {
                    id: "q2".to_string(),
                    question_type: "true_false".to_string(),
                    prompt: "Rust is a systems programming language".to_string(),
                    code_snippet: None,
                    options: vec![
                        QuestionOption {
                            id: "true".to_string(),
                            text: "True".to_string(),
                        },
                        QuestionOption {
                            id: "false".to_string(),
                            text: "False".to_string(),
                        },
                    ],
                    correct_answer: "true".to_string(),
                    explanation: "Rust is indeed a systems programming language".to_string(),
                    points: 10,
                },
            ],
        }
    }

    #[test]
    fn test_perfect_score() {
        let quiz = create_test_quiz();
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), "b".to_string());
        answers.insert("q2".to_string(), "true".to_string());

        let (score, correct, total) = grade_quiz(&quiz, &answers);
        assert_eq!(score, 20);
        assert_eq!(correct, 2);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_partial_score() {
        let quiz = create_test_quiz();
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), "b".to_string()); // Correct
        answers.insert("q2".to_string(), "false".to_string()); // Wrong

        let (score, correct, total) = grade_quiz(&quiz, &answers);
        assert_eq!(score, 10);
        assert_eq!(correct, 1);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_zero_score() {
        let quiz = create_test_quiz();
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), "a".to_string()); // Wrong
        answers.insert("q2".to_string(), "false".to_string()); // Wrong

        let (score, correct, total) = grade_quiz(&quiz, &answers);
        assert_eq!(score, 0);
        assert_eq!(correct, 0);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_missing_answers() {
        let quiz = create_test_quiz();
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), "b".to_string()); // Only answer q1

        let (score, correct, total) = grade_quiz(&quiz, &answers);
        assert_eq!(score, 10); // Only q1 counted
        assert_eq!(correct, 1);
        assert_eq!(total, 2); // But quiz has 2 questions
    }
}
