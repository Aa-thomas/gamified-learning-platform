import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';

interface QuizData {
  id: string;
  title: string;
  description: string;
  difficulty: string;
  passing_score: number;
  questions: Question[];
}

interface Question {
  id: string;
  prompt: string;
  code_snippet?: string;
  options: QuestionOption[];
  points: number;
}

interface QuestionOption {
  id: string;
  text: string;
}

interface QuizResult {
  score: number;
  total: number;
  score_percentage: number;
  passed: boolean;
  xp_earned: number;
  attempt_number: number;
  mastery_updates: Record<string, number>;
  feedback: QuestionFeedback[];
}

interface QuestionFeedback {
  question_id: string;
  user_answer?: string;
  correct_answer: string;
  is_correct: boolean;
  explanation: string;
}

export function Quiz() {
  const { quizId } = useParams<{ quizId: string }>();
  const navigate = useNavigate();
  const [quiz, setQuiz] = useState<QuizData | null>(null);
  const [loading, setLoading] = useState(true);
  const [currentQuestion, setCurrentQuestion] = useState(0);
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [submitted, setSubmitted] = useState(false);
  const [result, setResult] = useState<QuizResult | null>(null);
  const [startTime] = useState(Date.now());

  useEffect(() => {
    loadQuiz();
  }, []);

  const loadQuiz = async () => {
    try {
      const data = await invoke<QuizData>('load_quiz', { quizId });
      setQuiz(data);
      setLoading(false);
    } catch (error) {
      console.error('Failed to load quiz:', error);
      setLoading(false);
    }
  };

  const handleAnswerSelect = (questionId: string, optionId: string) => {
    setAnswers({ ...answers, [questionId]: optionId });
  };

  const handleSubmit = async () => {
    if (!quiz) return;

    // Check all questions answered
    const unanswered = quiz.questions.filter(q => !answers[q.id]);
    if (unanswered.length > 0) {
      alert(`Please answer all questions. ${unanswered.length} unanswered.`);
      return;
    }

    setSubmitted(true);
    const timeSpent = Date.now() - startTime;

    try {
      const result = await invoke<QuizResult>('submit_quiz', {
        request: {
          quiz_id: quizId,
          answers,
          time_spent_ms: timeSpent,
        },
      });
      setResult(result);
    } catch (error) {
      console.error('Failed to submit quiz:', error);
      alert('Failed to submit quiz');
      setSubmitted(false);
    }
  };

  if (loading) {
    return <div className="p-8">Loading quiz...</div>;
  }

  if (!quiz) {
    return <div className="p-8">Quiz not found</div>;
  }

  if (result) {
    return (
      <div className="max-w-4xl mx-auto p-8">
        <div className="bg-white rounded-lg shadow-lg p-8">
          <h1 className="text-3xl font-bold mb-6">Quiz Results</h1>
          
          <div className="mb-8">
            <div className="text-6xl font-bold text-center mb-4">
              {Math.round(result.score_percentage)}%
            </div>
            <div className="text-center text-gray-600 mb-2">
              {result.score} / {result.total} points
            </div>
            {result.passed ? (
              <div className="text-center text-green-600 font-bold text-xl">
                ✓ PASSED
              </div>
            ) : (
              <div className="text-center text-red-600 font-bold text-xl">
                ✗ FAILED (Need {quiz.passing_score}% to pass)
              </div>
            )}
          </div>

          <div className="mb-8 p-4 bg-blue-50 rounded">
            <div className="text-lg font-bold mb-2">XP Earned: {result.xp_earned}</div>
            {result.attempt_number > 1 && (
              <div className="text-sm text-gray-600">
                Attempt #{result.attempt_number} (reduced XP)
              </div>
            )}
          </div>

          <div className="mb-8">
            <h2 className="text-xl font-bold mb-4">Question Review</h2>
            {result.feedback.map((fb, index) => (
              <div
                key={fb.question_id}
                className={`mb-4 p-4 rounded border-2 ${
                  fb.is_correct ? 'border-green-300 bg-green-50' : 'border-red-300 bg-red-50'
                }`}
              >
                <div className="flex items-center mb-2">
                  <span className="font-bold mr-2">Question {index + 1}</span>
                  {fb.is_correct ? (
                    <span className="text-green-600">✓ Correct</span>
                  ) : (
                    <span className="text-red-600">✗ Incorrect</span>
                  )}
                </div>
                {!fb.is_correct && (
                  <div className="text-sm mb-2">
                    <div>Your answer: {fb.user_answer}</div>
                    <div>Correct answer: {fb.correct_answer}</div>
                  </div>
                )}
                <div className="text-sm text-gray-700">{fb.explanation}</div>
              </div>
            ))}
          </div>

          <div className="flex gap-4">
            {!result.passed && (
              <button
                onClick={() => {
                  setSubmitted(false);
                  setResult(null);
                  setAnswers({});
                  setCurrentQuestion(0);
                }}
                className="px-6 py-2 bg-orange-600 text-white rounded hover:bg-orange-700"
              >
                Retake Quiz
              </button>
            )}
            <button
              onClick={() => navigate('/')}
              className="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    );
  }

  const question = quiz.questions[currentQuestion];
  const allAnswered = quiz.questions.every(q => answers[q.id]);

  return (
    <div className="max-w-4xl mx-auto p-8">
      <div className="mb-6">
        <button
          onClick={() => navigate('/')}
          className="text-blue-600 hover:text-blue-800"
        >
          ← Back
        </button>
      </div>

      <div className="mb-6">
        <h1 className="text-3xl font-bold mb-2">{quiz.title}</h1>
        <p className="text-gray-600 mb-4">{quiz.description}</p>
        <div className="flex gap-4 text-sm text-gray-600">
          <span>Difficulty: {quiz.difficulty}</span>
          <span>Passing Score: {quiz.passing_score}%</span>
        </div>
      </div>

      <div className="mb-6">
        <div className="w-full bg-gray-200 rounded-full h-2">
          <div
            className="bg-blue-600 h-2 rounded-full transition-all"
            style={{ width: `${((currentQuestion + 1) / quiz.questions.length) * 100}%` }}
          />
        </div>
        <p className="text-sm text-gray-600 mt-2">
          Question {currentQuestion + 1} of {quiz.questions.length}
        </p>
      </div>

      <div className="bg-white rounded-lg shadow p-8 mb-6">
        <h2 className="text-xl font-bold mb-4">{question.prompt}</h2>
        
        {question.code_snippet && (
          <pre className="bg-gray-100 p-4 rounded mb-4 overflow-x-auto">
            <code>{question.code_snippet}</code>
          </pre>
        )}

        <div className="space-y-3">
          {question.options.map((option) => (
            <label
              key={option.id}
              className={`block p-4 border-2 rounded cursor-pointer transition-colors ${
                answers[question.id] === option.id
                  ? 'border-blue-600 bg-blue-50'
                  : 'border-gray-300 hover:border-blue-400'
              }`}
            >
              <input
                type="radio"
                name={`question-${question.id}`}
                value={option.id}
                checked={answers[question.id] === option.id}
                onChange={() => handleAnswerSelect(question.id, option.id)}
                className="mr-3"
              />
              {option.text}
            </label>
          ))}
        </div>
      </div>

      <div className="flex justify-between items-center">
        <button
          onClick={() => setCurrentQuestion(Math.max(0, currentQuestion - 1))}
          disabled={currentQuestion === 0}
          className="px-4 py-2 text-gray-700 hover:text-gray-900 disabled:text-gray-400"
        >
          ← Previous
        </button>

        <div className="flex gap-2">
          {quiz.questions.map((_, index) => (
            <button
              key={index}
              onClick={() => setCurrentQuestion(index)}
              className={`w-8 h-8 rounded ${
                index === currentQuestion
                  ? 'bg-blue-600 text-white'
                  : answers[quiz.questions[index].id]
                  ? 'bg-green-300'
                  : 'bg-gray-300'
              }`}
            >
              {index + 1}
            </button>
          ))}
        </div>

        {currentQuestion < quiz.questions.length - 1 ? (
          <button
            onClick={() => setCurrentQuestion(currentQuestion + 1)}
            className="px-4 py-2 text-gray-700 hover:text-gray-900"
          >
            Next →
          </button>
        ) : (
          <button
            onClick={handleSubmit}
            disabled={!allAnswered || submitted}
            className={`px-6 py-2 rounded ${
              allAnswered && !submitted
                ? 'bg-green-600 hover:bg-green-700 text-white'
                : 'bg-gray-300 text-gray-500 cursor-not-allowed'
            }`}
          >
            {submitted ? 'Submitting...' : 'Submit Quiz'}
          </button>
        )}
      </div>
    </div>
  );
}
