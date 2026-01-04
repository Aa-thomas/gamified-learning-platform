import { useEffect } from 'react'
import { Download, X, RefreshCw } from 'lucide-react'
import { useUpdateStore } from '@/stores/updateStore'
import { Button } from './common/Button'

export function UpdateChecker() {
  const {
    updateAvailable,
    updateInfo,
    downloading,
    error,
    checkForUpdate,
    downloadUpdate,
    dismissUpdate,
    clearError,
  } = useUpdateStore()

  // Check for updates on mount
  useEffect(() => {
    checkForUpdate()
  }, [])

  if (error) {
    return (
      <div className="fixed bottom-4 right-4 max-w-sm bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg shadow-lg p-4 z-50">
        <div className="flex items-start gap-3">
          <div className="flex-1">
            <p className="text-sm font-medium text-red-800 dark:text-red-200">
              Update check failed
            </p>
            <p className="text-xs text-red-600 dark:text-red-400 mt-1">
              {error}
            </p>
          </div>
          <button
            onClick={clearError}
            className="text-red-400 hover:text-red-600 dark:hover:text-red-300"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
        <Button
          variant="outline"
          size="sm"
          className="mt-2 w-full"
          onClick={checkForUpdate}
        >
          <RefreshCw className="w-3 h-3 mr-1" />
          Retry
        </Button>
      </div>
    )
  }

  if (!updateAvailable || !updateInfo) {
    return null
  }

  return (
    <div className="fixed bottom-4 right-4 max-w-sm bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4 z-50">
      <div className="flex items-start gap-3">
        <div className="w-10 h-10 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0">
          <Download className="w-5 h-5 text-primary" />
        </div>
        <div className="flex-1">
          <h3 className="text-sm font-semibold text-gray-900 dark:text-white">
            Update Available
          </h3>
          <p className="text-xs text-gray-600 dark:text-gray-400 mt-0.5">
            Version {updateInfo.version} is ready to install
          </p>
          {updateInfo.body && (
            <p className="text-xs text-gray-500 dark:text-gray-500 mt-2 line-clamp-2">
              {updateInfo.body}
            </p>
          )}
        </div>
        <button
          onClick={dismissUpdate}
          className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="flex gap-2 mt-3">
        <Button
          variant="outline"
          size="sm"
          className="flex-1"
          onClick={dismissUpdate}
          disabled={downloading}
        >
          Later
        </Button>
        <Button
          size="sm"
          className="flex-1"
          onClick={downloadUpdate}
          loading={downloading}
        >
          {downloading ? 'Updating...' : 'Update Now'}
        </Button>
      </div>

      {downloading && (
        <div className="mt-3">
          <div className="h-1 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all duration-300"
              style={{ width: '100%' }}
            />
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1 text-center">
            Downloading and installing...
          </p>
        </div>
      )}
    </div>
  )
}
