import { useState } from 'react'
import { save, open } from '@tauri-apps/plugin-dialog'
import { useUserStore } from '@/stores/userStore'
import { useSystemStore } from '@/stores/systemStore'
import { useThemeStore, Theme } from '@/stores/themeStore'
import { Button } from '@/components/common/Button'
import { Sun, Moon, Monitor, Download, Upload, Trash2 } from 'lucide-react'

const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'system', label: 'System', icon: Monitor },
]

export function Settings() {
  const { user } = useUserStore()
  const { exportUserData, importUserData, resetAllProgress, loading } = useSystemStore()
  const { theme, setTheme } = useThemeStore()
  const [showResetConfirm, setShowResetConfirm] = useState(false)
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)

  const handleExport = async () => {
    try {
      const path = await save({
        defaultPath: 'glp-backup.json',
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path) {
        await exportUserData(path)
        setMessage({ type: 'success', text: 'Progress exported successfully!' })
      }
    } catch (error) {
      setMessage({ type: 'error', text: `Export failed: ${error}` })
    }
  }

  const handleImport = async () => {
    try {
      const path = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path && typeof path === 'string') {
        await importUserData(path)
        setMessage({ type: 'success', text: 'Progress imported successfully! Refresh to see changes.' })
      }
    } catch (error) {
      setMessage({ type: 'error', text: `Import failed: ${error}` })
    }
  }

  const handleReset = async () => {
    try {
      await resetAllProgress()
      setShowResetConfirm(false)
      setMessage({ type: 'success', text: 'Progress reset successfully!' })
    } catch (error) {
      setMessage({ type: 'error', text: `Reset failed: ${error}` })
    }
  }

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-6 dark:text-white">Settings</h1>

      {message && (
        <div
          className={`mb-4 p-4 rounded-lg ${
            message.type === 'success'
              ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300'
              : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300'
          }`}
        >
          {message.text}
          <button
            onClick={() => setMessage(null)}
            className="float-right font-bold"
          >
            ×
          </button>
        </div>
      )}

      <div className="bg-white dark:bg-gray-800 rounded-lg shadow divide-y dark:divide-gray-700">
        {/* Appearance Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4 dark:text-white">Appearance</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Theme
              </label>
              <div className="flex gap-2">
                {themeOptions.map((option) => {
                  const Icon = option.icon
                  return (
                    <button
                      key={option.value}
                      onClick={() => setTheme(option.value)}
                      className={`flex items-center gap-2 px-4 py-2 rounded-lg border transition-colors ${
                        theme === option.value
                          ? 'border-primary bg-primary/10 text-primary'
                          : 'border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                      {option.label}
                    </button>
                  )
                })}
              </div>
            </div>
          </div>
        </div>

        {/* Profile Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4 dark:text-white">Profile</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                User ID
              </label>
              <input
                type="text"
                value={user?.id ?? ''}
                disabled
                className="w-full px-4 py-2 bg-gray-50 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-500 dark:text-gray-400"
              />
            </div>
          </div>
        </div>

        {/* Data Management Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4 dark:text-white">Data Management</h2>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            All your progress is stored locally on your device. You can export your data for backup
            or import a previous backup.
          </p>
          <div className="flex flex-wrap gap-3">
            <Button
              variant="outline"
              onClick={handleExport}
              loading={loading}
              disabled={!user}
            >
              <Download className="w-4 h-4 mr-2" />
              Export Progress
            </Button>
            <Button
              variant="outline"
              onClick={handleImport}
              loading={loading}
            >
              <Upload className="w-4 h-4 mr-2" />
              Import Progress
            </Button>
          </div>
        </div>

        {/* Danger Zone */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4 text-red-600 dark:text-red-400">Danger Zone</h2>
          {!showResetConfirm ? (
            <Button
              variant="danger"
              onClick={() => setShowResetConfirm(true)}
              disabled={!user}
            >
              <Trash2 className="w-4 h-4 mr-2" />
              Reset All Progress
            </Button>
          ) : (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <p className="text-sm text-red-700 dark:text-red-300 mb-4">
                Are you sure? This will permanently delete all your progress, including XP, badges,
                and quiz attempts. This cannot be undone.
              </p>
              <div className="flex gap-3">
                <Button
                  variant="danger"
                  onClick={handleReset}
                  loading={loading}
                >
                  Yes, Reset Everything
                </Button>
                <Button
                  variant="outline"
                  onClick={() => setShowResetConfirm(false)}
                >
                  Cancel
                </Button>
              </div>
            </div>
          )}
        </div>

        {/* About Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4 dark:text-white">About</h2>
          <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
            <p>
              <span className="font-medium">App Version:</span> 0.1.0
            </p>
            <p>
              <span className="font-medium">Built with:</span> Tauri + React + Rust
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
