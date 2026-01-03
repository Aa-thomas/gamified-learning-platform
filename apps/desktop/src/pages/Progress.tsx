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

  // Calculate XP needed for next level using the formula: 100 * N^1.5
  const xpForLevel = (level: number) => Math.floor(100 * Math.pow(level, 1.5))
  const xpForCurrentLevel = xpForLevel(user.level)
  const xpForNextLevel = xpForLevel(user.level + 1)
  const xpProgress = user.xp - xpForCurrentLevel
  const xpNeeded = xpForNextLevel - xpForCurrentLevel
  const levelProgress = Math.min((xpProgress / xpNeeded) * 100, 100)

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Your Progress</h1>

      {/* Level Progress */}
      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 bg-yellow-100 rounded-full flex items-center justify-center">
              <Trophy className="text-yellow-600" size={24} />
            </div>
            <div>
              <p className="text-sm text-gray-500">Current Level</p>
              <p className="text-2xl font-bold">Level {user.level}</p>
            </div>
          </div>
          <div className="text-right">
            <p className="text-sm text-gray-500">XP to Next Level</p>
            <p className="text-lg font-semibold">
              {xpProgress.toLocaleString()} / {xpNeeded.toLocaleString()}
            </p>
          </div>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-3">
          <div
            className="bg-yellow-500 h-3 rounded-full transition-all"
            style={{ width: `${levelProgress}%` }}
          />
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="bg-white rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-blue-600 mb-2">
            <Target size={20} />
            <span className="text-sm font-medium">Completed</span>
          </div>
          <p className="text-2xl font-bold">{completedNodes.length}</p>
          <p className="text-xs text-gray-500">lessons</p>
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-yellow-600 mb-2">
            <Clock size={20} />
            <span className="text-sm font-medium">In Progress</span>
          </div>
          <p className="text-2xl font-bold">{inProgressNodes.length}</p>
          <p className="text-xs text-gray-500">lessons</p>
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-green-600 mb-2">
            <Trophy size={20} />
            <span className="text-sm font-medium">Total XP</span>
          </div>
          <p className="text-2xl font-bold">{totalXpEarned.toLocaleString()}</p>
          <p className="text-xs text-gray-500">earned</p>
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <div className="flex items-center gap-2 text-orange-600 mb-2">
            <Flame size={20} />
            <span className="text-sm font-medium">Streak</span>
          </div>
          <p className="text-2xl font-bold">{user.streak_days}</p>
          <p className="text-xs text-gray-500">days</p>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold mb-4">Statistics</h2>
        <div className="space-y-3">
          <div className="flex justify-between py-2 border-b border-gray-100">
            <span className="text-gray-600">Total Attempts</span>
            <span className="font-medium">{totalAttempts}</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-100">
            <span className="text-gray-600">Account Created</span>
            <span className="font-medium">
              {new Date(user.created_at).toLocaleDateString()}
            </span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-100">
            <span className="text-gray-600">Last Active</span>
            <span className="font-medium">
              {new Date(user.last_active_date).toLocaleDateString()}
            </span>
          </div>
          <div className="flex justify-between py-2">
            <span className="text-gray-600">Average XP per Lesson</span>
            <span className="font-medium">
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
