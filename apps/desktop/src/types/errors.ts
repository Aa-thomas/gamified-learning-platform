/**
 * Error codes for categorizing application errors
 */
export type ErrorCode =
  | 'DOCKER_NOT_RUNNING'
  | 'DOCKER_NOT_INSTALLED'
  | 'LLM_API_TIMEOUT'
  | 'LLM_RATE_LIMITED'
  | 'LLM_API_KEY_INVALID'
  | 'LLM_API_KEY_MISSING'
  | 'DATABASE_LOCKED'
  | 'DATABASE_CORRUPTED'
  | 'CODE_TIMEOUT'
  | 'CODE_MEMORY_EXCEEDED'
  | 'CONTENT_NOT_FOUND'
  | 'CURRICULUM_NOT_LOADED'
  | 'NETWORK_ERROR'
  | 'VALIDATION_ERROR'
  | 'UNKNOWN'

/**
 * Structured application error with user-friendly messaging
 */
export interface AppError {
  /** Machine-readable error code */
  code: ErrorCode
  /** Technical error message (for logging) */
  message: string
  /** User-friendly error message */
  userMessage: string
  /** Suggested recovery action */
  recoveryAction?: string
  /** Whether the operation can be retried */
  retryable: boolean
}

/**
 * Error response from backend commands
 */
export interface CommandError {
  code: string
  message: string
  details?: string
}
