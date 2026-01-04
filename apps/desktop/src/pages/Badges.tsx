import { useEffect, useState } from 'react'
import { Trophy, Lock, Check, Star, Flame, Target, BookOpen, Zap } from 'lucide-react'
import { useBadgeStore, BadgeWithProgress } from '@/stores/badgeStore'

export function Badges() {
  const { badges, loading, fetchAllBadges } = useBadgeStore()
  const [selectedBadge, setSelectedBadge] = useState<BadgeWithProgress | null>(null)

  useEffect(() => {
    fetchAllBadges()
  }, [fetchAllBadges])

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'Streak':
        return <Flame className="w-4 h-4" />
      case 'Level':
        return <Star className="w-4 h-4" />
      case 'Xp':
        return <Zap className="w-4 h-4" />
      case 'Completion':
        return <BookOpen className="w-4 h-4" />
      case 'Mastery':
        return <Target className="w-4 h-4" />
      default:
        return <Trophy className="w-4 h-4" />
    }
  }

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'Streak':
        return 'text-orange-500 bg-orange-50'
      case 'Level':
        return 'text-yellow-500 bg-yellow-50'
      case 'Xp':
        return 'text-purple-500 bg-purple-50'
      case 'Completion':
        return 'text-blue-500 bg-blue-50'
      case 'Mastery':
        return 'text-green-500 bg-green-50'
      default:
        return 'text-gray-500 bg-gray-50'
    }
  }

  const earnedBadges = badges.filter((b) => b.is_earned)
  const lockedBadges = badges.filter((b) => !b.is_earned)

  if (loading && badges.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Loading badges...</p>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="mb-8">
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Trophy className="w-7 h-7 text-yellow-500" />
          Badge Collection
        </h1>
        <p className="text-gray-500 mt-1">
          {earnedBadges.length} of {badges.length} badges earned
        </p>
      </div>

      {/* Earned Badges */}
      <section className="mb-8">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Check className="w-5 h-5 text-green-500" />
          Earned Badges ({earnedBadges.length})
        </h2>
        {earnedBadges.length === 0 ? (
          <p className="text-gray-400 text-sm">
            Complete activities to earn your first badge!
          </p>
        ) : (
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {earnedBadges.map((badge) => (
              <button
                key={badge.definition.id}
                onClick={() => setSelectedBadge(badge)}
                className="bg-white rounded-xl p-4 shadow-sm border-2 border-yellow-200 
                         hover:border-yellow-400 transition-all hover:shadow-md text-left"
              >
                <div className="text-4xl mb-2">{badge.definition.icon}</div>
                <h3 className="font-semibold text-gray-800">
                  {badge.definition.name}
                </h3>
                <div
                  className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full mt-2 ${getCategoryColor(badge.definition.category)}`}
                >
                  {getCategoryIcon(badge.definition.category)}
                  {badge.definition.category}
                </div>
                {badge.earned_at && (
                  <p className="text-xs text-gray-400 mt-2">
                    Earned {new Date(badge.earned_at).toLocaleDateString()}
                  </p>
                )}
              </button>
            ))}
          </div>
        )}
      </section>

      {/* Locked Badges */}
      <section>
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Lock className="w-5 h-5 text-gray-400" />
          Locked Badges ({lockedBadges.length})
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {lockedBadges.map((badge) => (
            <button
              key={badge.definition.id}
              onClick={() => setSelectedBadge(badge)}
              className="bg-gray-50 rounded-xl p-4 border-2 border-gray-100 
                       hover:border-gray-200 transition-all text-left opacity-75 hover:opacity-100"
            >
              <div className="text-4xl mb-2 grayscale">
                {badge.definition.icon}
              </div>
              <h3 className="font-semibold text-gray-600">
                {badge.definition.name}
              </h3>
              <div
                className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full mt-2 ${getCategoryColor(badge.definition.category)}`}
              >
                {getCategoryIcon(badge.definition.category)}
                {badge.definition.category}
              </div>
              {/* Progress bar */}
              <div className="mt-3">
                <div className="flex justify-between text-xs text-gray-500 mb-1">
                  <span>Progress</span>
                  <span>{Math.round(badge.progress * 100)}%</span>
                </div>
                <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary rounded-full transition-all"
                    style={{ width: `${badge.progress * 100}%` }}
                  />
                </div>
              </div>
            </button>
          ))}
        </div>
      </section>

      {/* Badge Detail Modal */}
      {selectedBadge && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
          onClick={() => setSelectedBadge(null)}
        >
          <div
            className="bg-white rounded-2xl p-6 max-w-md w-full mx-4 shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="text-center">
              <div
                className={`text-6xl mb-4 ${!selectedBadge.is_earned ? 'grayscale' : ''}`}
              >
                {selectedBadge.definition.icon}
              </div>
              <h2 className="text-xl font-bold">
                {selectedBadge.definition.name}
              </h2>
              <p className="text-gray-500 mt-2">
                {selectedBadge.definition.description}
              </p>

              <div
                className={`inline-flex items-center gap-1 text-sm px-3 py-1 rounded-full mt-4 ${getCategoryColor(selectedBadge.definition.category)}`}
              >
                {getCategoryIcon(selectedBadge.definition.category)}
                {selectedBadge.definition.category}
              </div>

              {selectedBadge.is_earned ? (
                <div className="mt-6 p-4 bg-green-50 rounded-lg">
                  <div className="flex items-center justify-center gap-2 text-green-600 font-semibold">
                    <Check className="w-5 h-5" />
                    Badge Earned!
                  </div>
                  {selectedBadge.earned_at && (
                    <p className="text-sm text-green-500 mt-1">
                      {new Date(selectedBadge.earned_at).toLocaleDateString(
                        'en-US',
                        {
                          year: 'numeric',
                          month: 'long',
                          day: 'numeric',
                        }
                      )}
                    </p>
                  )}
                </div>
              ) : (
                <div className="mt-6">
                  <div className="flex justify-between text-sm text-gray-600 mb-2">
                    <span>
                      {selectedBadge.current_value.toLocaleString()} /{' '}
                      {selectedBadge.definition.threshold.toLocaleString()}
                    </span>
                    <span>{Math.round(selectedBadge.progress * 100)}%</span>
                  </div>
                  <div className="h-3 bg-gray-200 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-primary rounded-full transition-all"
                      style={{ width: `${selectedBadge.progress * 100}%` }}
                    />
                  </div>
                </div>
              )}

              <button
                onClick={() => setSelectedBadge(null)}
                className="mt-6 px-6 py-2 bg-gray-100 hover:bg-gray-200 rounded-lg 
                         font-medium transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
