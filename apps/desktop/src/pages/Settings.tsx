import { useUserStore } from '@/stores/userStore'

export function Settings() {
  const { user } = useUserStore()

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Settings</h1>

      <div className="bg-white rounded-lg shadow divide-y">
        {/* Profile Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4">Profile</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Username
              </label>
              <input
                type="text"
                value={user?.username ?? ''}
                disabled
                className="w-full px-4 py-2 bg-gray-50 border border-gray-300 rounded-lg text-gray-500"
              />
              <p className="text-xs text-gray-500 mt-1">
                Username cannot be changed.
              </p>
            </div>
          </div>
        </div>

        {/* About Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4">About</h2>
          <div className="space-y-2 text-sm text-gray-600">
            <p>
              <span className="font-medium">App Version:</span> 0.1.0
            </p>
            <p>
              <span className="font-medium">Built with:</span> Tauri + React +
              Rust
            </p>
          </div>
        </div>

        {/* Data Section */}
        <div className="p-6">
          <h2 className="text-lg font-semibold mb-4">Data</h2>
          <p className="text-sm text-gray-600 mb-4">
            All your progress is stored locally on your device. No data is sent
            to external servers.
          </p>
          <button
            className="px-4 py-2 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50 transition-colors"
            onClick={() => {
              // In a full implementation, this would reset user data
              alert('This feature is not yet implemented.')
            }}
          >
            Reset Progress
          </button>
        </div>
      </div>
    </div>
  )
}
