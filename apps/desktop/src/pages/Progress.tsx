import { useEffect } from 'react'
import { Trophy, Target, Clock, Flame } from 'lucide-react'
import { useUserStore } from '@/stores/userStore'
import { useProgressStore } from '@/stores/progressStore'

export function Progress() {
  const { user } = useUserStore()
  const { progress, fetchAllProgress } = useProgressStore()

  useEffect(() => {
    fetchAllProgress()
  }, [fetchAllProgress])

  if (!user) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Loading...</p>
      </div>
    )
  }

  const completedNodes = Object.values(progress).filter(
    (p) => p.status === 'completed'
  )
  const inProgressNodes = Object.values(progress).filter(
    (p) => p.status === 'in_progress'
  )
  const totalXpEarned = completedNodes.reduce((sum, p) => sum + p.xp_earned, 0)
  const totalAttempts = Object.values(progress).reduce(
    (sum, p) => sum + p.attempts,
    0
  )

  // Use pre-calculated values from backend
  const levelProgress = user.xp_progress_percentage

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-6 dark:text-white">Your Progress</h1>

      {/* Level Progress */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 bg-yellow-100 dark:bg-yellow-900/30 rounded-full flex items-center justify-center">
              <Trophy className="text-yellow-600" size={24} />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Current Level</p>
              <p className="text-2xl font-bold dark:text-white">Level {user.current_level}</p>
            </div>
          </div>
          <div className="text-right">
            <p className="text-sm text-gray-500 dark:text-gray-400">XP to Next Level</p>
            <p className="text-lg font-semibold dark:text-white">
              {user.total_xp.toLocaleString()} / {user.xp_for_next_level.toLocaleString()}
            </p>
          </div>
        </div>
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
          <div
            className="bg-yellow-500 h-3 rounded-full transition-all"
            style={{ width: `${levelProgress}%` }}
          />
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-blue-600 mb-2">
            <Target size={20} />
            <span className="text-sm font-medium">Completed</span>
          </div>
          <p className="text-2xl font-bold dark:text-white">{completedNodes.length}</p>
          <p className="text-xs text-gray-500 dark:text-gray-400">lessons</p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-yellow-600 mb-2">
            <Clock size={20} />
            <span className="text-sm font-medium">In Progress</span>
          </div>
          <p className="text-2xl font-bold dark:text-white">{inProgressNodes.length}</p>
          <p className="text-xs text-gray-500 dark:text-gray-400">lessons</p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-green-600 mb-2">
            <Trophy size={20} />
            <span className="text-sm font-medium">Total XP</span>
          </div>
          <p className="text-2xl font-bold dark:text-white">{totalXpEarned.toLocaleString()}</p>
          <p className="text-xs text-gray-500 dark:text-gray-400">earned</p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-orange-600 mb-2">
            <Flame size={20} />
            <span className="text-sm font-medium">Streak</span>
          </div>
          <p className="text-2xl font-bold dark:text-white">{user.current_streak}</p>
          <p className="text-xs text-gray-500 dark:text-gray-400">days</p>
        </div>
      </div>

      {/* Statistics */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold mb-4 dark:text-white">Statistics</h2>
        <div className="space-y-3">
          <div className="flex justify-between py-2 border-b border-gray-100 dark:border-gray-700">
            <span className="text-gray-600 dark:text-gray-400">Total Attempts</span>
            <span className="font-medium dark:text-white">{totalAttempts}</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-100 dark:border-gray-700">
            <span className="text-gray-600 dark:text-gray-400">Total XP Earned</span>
            <span className="font-medium dark:text-white">{user.total_xp.toLocaleString()}</span>
          </div>
          <div className="flex justify-between py-2">
            <span className="text-gray-600 dark:text-gray-400">Average XP per Lesson</span>
            <span className="font-medium dark:text-white">
              {completedNodes.length > 0
                ? Math.round(totalXpEarned / completedNodes.length)
                : 0}
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}
