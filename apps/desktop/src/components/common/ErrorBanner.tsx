import { AlertCircle, X, RefreshCw } from 'lucide-react'
import { AppError } from '../../types/errors'

interface ErrorBannerProps {
  error: AppError
  onDismiss: () => void
  onRetry?: () => void
}

export function ErrorBanner({ error, onDismiss, onRetry }: ErrorBannerProps) {
  return (
    <div
      className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-4"
      role="alert"
    >
      <div className="flex items-start gap-3">
        <AlertCircle className="w-5 h-5 text-red-500 dark:text-red-400 flex-shrink-0 mt-0.5" />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-red-800 dark:text-red-200">
            {error.userMessage}
          </p>
          {error.recoveryAction && (
            <p className="text-sm text-red-600 dark:text-red-300 mt-1">
              {error.recoveryAction}
            </p>
          )}
        </div>
        <div className="flex items-center gap-2 flex-shrink-0">
          {error.retryable && onRetry && (
            <button
              onClick={onRetry}
              className="p-1.5 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-800/50 rounded transition-colors"
              aria-label="Retry"
            >
              <RefreshCw className="w-4 h-4" />
            </button>
          )}
          <button
            onClick={onDismiss}
            className="p-1.5 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-800/50 rounded transition-colors"
            aria-label="Dismiss"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  )
}
