import { useEffect } from 'react'
import { Flame, Zap, Trophy } from 'lucide-react'
import { useUserStore } from '@/stores/userStore'

export function StatusBar() {
  const { user, fetchUser } = useUserStore()

  useEffect(() => {
    fetchUser()
  }, [fetchUser])

  if (!user) {
    return (
      <footer className="bg-gray-50 border-t border-gray-200 px-4 py-2">
        <div className="flex items-center justify-center text-sm text-gray-500">
          Loading user data...
        </div>
      </footer>
    )
  }

  return (
    <footer className="bg-gray-50 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-4 py-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1 text-sm">
            <Trophy size={16} className="text-yellow-500" />
            <span className="font-medium dark:text-white">Level {user.current_level}</span>
          </div>

          <div className="flex items-center gap-1 text-sm">
            <Zap size={16} className="text-blue-500" />
            <span className="dark:text-gray-300">{user.total_xp.toLocaleString()} XP</span>
          </div>

          <div className="flex items-center gap-1 text-sm">
            <Flame size={16} className="text-orange-500" />
            <span className="dark:text-gray-300">{user.current_streak} day streak</span>
          </div>
        </div>

        <div className="text-sm text-gray-500 dark:text-gray-400">
          {Math.round(user.xp_progress_percentage)}% to next level
        </div>
      </div>
    </footer>
  )
}
