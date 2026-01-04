import { AppError, ErrorCode } from '../types/errors'

/**
 * Error pattern matching rules
 */
const ERROR_PATTERNS: Array<{
  pattern: RegExp
  code: ErrorCode
  userMessage: string
  recoveryAction?: string
  retryable: boolean
}> = [
  {
    pattern: /docker is not installed/i,
    code: 'DOCKER_NOT_INSTALLED',
    userMessage: 'Docker is not installed on your system.',
    recoveryAction: 'Install Docker to run code challenges.',
    retryable: false,
  },
  {
    pattern: /docker is not (running|available)/i,
    code: 'DOCKER_NOT_RUNNING',
    userMessage: 'Docker is not running.',
    recoveryAction: 'Start Docker Desktop or the Docker service.',
    retryable: true,
  },
  {
    pattern: /timeout (exceeded|after)/i,
    code: 'CODE_TIMEOUT',
    userMessage: 'Your code took too long to execute.',
    recoveryAction: 'Check for infinite loops or optimize your code.',
    retryable: true,
  },
  {
    pattern: /memory (exceeded|limit|bomb)/i,
    code: 'CODE_MEMORY_EXCEEDED',
    userMessage: 'Your code used too much memory.',
    recoveryAction: 'Check for memory leaks or reduce data size.',
    retryable: true,
  },
  {
    pattern: /rate limit/i,
    code: 'LLM_RATE_LIMITED',
    userMessage: 'API rate limit reached. Please wait a moment.',
    recoveryAction: 'Wait a few seconds and try again.',
    retryable: true,
  },
  {
    pattern: /api.*timeout|request.*timeout/i,
    code: 'LLM_API_TIMEOUT',
    userMessage: 'The grading service is taking too long to respond.',
    recoveryAction: 'Check your internet connection and try again.',
    retryable: true,
  },
  {
    pattern: /invalid.*api.*key|api.*key.*invalid|unauthorized/i,
    code: 'LLM_API_KEY_INVALID',
    userMessage: 'Your OpenAI API key is invalid.',
    recoveryAction: 'Check your API key in Settings.',
    retryable: false,
  },
  {
    pattern: /api.*key.*missing|no.*api.*key/i,
    code: 'LLM_API_KEY_MISSING',
    userMessage: 'OpenAI API key is not configured.',
    recoveryAction: 'Add your API key in Settings.',
    retryable: false,
  },
  {
    pattern: /database.*locked|sqlite.*locked/i,
    code: 'DATABASE_LOCKED',
    userMessage: 'Database is temporarily busy.',
    recoveryAction: 'Wait a moment and try again.',
    retryable: true,
  },
  {
    pattern: /database.*corrupt|malformed/i,
    code: 'DATABASE_CORRUPTED',
    userMessage: 'Database file is corrupted.',
    recoveryAction: 'Try restoring from a backup in Settings.',
    retryable: false,
  },
  {
    pattern: /content.*not.*found|file.*not.*found/i,
    code: 'CONTENT_NOT_FOUND',
    userMessage: 'The requested content could not be found.',
    recoveryAction: 'Try reloading the curriculum.',
    retryable: true,
  },
  {
    pattern: /no.*curriculum.*loaded|curriculum.*not.*active/i,
    code: 'CURRICULUM_NOT_LOADED',
    userMessage: 'No curriculum is currently active.',
    recoveryAction: 'Import or select a curriculum first.',
    retryable: false,
  },
  {
    pattern: /network|connection.*refused|fetch.*failed/i,
    code: 'NETWORK_ERROR',
    userMessage: 'Network connection failed.',
    recoveryAction: 'Check your internet connection.',
    retryable: true,
  },
  {
    pattern: /validation|invalid.*data|required.*field/i,
    code: 'VALIDATION_ERROR',
    userMessage: 'The provided data is invalid.',
    recoveryAction: 'Check your input and try again.',
    retryable: false,
  },
]

/**
 * Parse a raw error message into a structured AppError
 */
export function parseError(error: unknown): AppError {
  const message = getErrorMessage(error)

  // Try to match against known patterns
  for (const rule of ERROR_PATTERNS) {
    if (rule.pattern.test(message)) {
      return {
        code: rule.code,
        message,
        userMessage: rule.userMessage,
        recoveryAction: rule.recoveryAction,
        retryable: rule.retryable,
      }
    }
  }

  // Unknown error
  return {
    code: 'UNKNOWN',
    message,
    userMessage: 'An unexpected error occurred.',
    recoveryAction: 'Try again or restart the application.',
    retryable: true,
  }
}

/**
 * Extract error message from various error types
 */
function getErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error
  }
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'object' && error !== null) {
    if ('message' in error && typeof error.message === 'string') {
      return error.message
    }
    return JSON.stringify(error)
  }
  return String(error)
}

/**
 * Retry a function with exponential backoff
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  options: {
    maxAttempts?: number
    initialDelay?: number
    maxDelay?: number
    exponential?: boolean
  } = {}
): Promise<T> {
  const {
    maxAttempts = 3,
    initialDelay = 1000,
    maxDelay = 10000,
    exponential = true,
  } = options

  let lastError: unknown
  let delay = initialDelay

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error
      const parsed = parseError(error)

      // Don't retry non-retryable errors
      if (!parsed.retryable || attempt === maxAttempts) {
        throw error
      }

      // Wait before retrying
      await new Promise((resolve) => setTimeout(resolve, delay))

      // Increase delay for next attempt
      if (exponential) {
        delay = Math.min(delay * 2, maxDelay)
      }
    }
  }

  throw lastError
}
