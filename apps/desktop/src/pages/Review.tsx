import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { 
  RotateCcw, 
  Calendar, 
  Clock, 
  CheckCircle2, 
  AlertCircle,
  ChevronRight,
  Brain
} from 'lucide-react'
import { useReviewStore, ReviewItem } from '@/stores/reviewStore'
import { useContentStore } from '@/stores/contentStore'

export function Review() {
  const navigate = useNavigate()
  const { dueReviews, dueCount, loading, fetchDueReviews, applyDecayOnStartup } = useReviewStore()
  const { tree, fetchContentTree } = useContentStore()
  const [decayedSkills, setDecayedSkills] = useState(0)

  useEffect(() => {
    // Apply mastery decay on page load
    applyDecayOnStartup().then(setDecayedSkills)
    fetchDueReviews()
    fetchContentTree()
  }, [applyDecayOnStartup, fetchDueReviews, fetchContentTree])

  const getQuizTitle = (quizId: string): string => {
    if (!tree) return quizId
    
    for (const week of tree.weeks) {
      for (const day of week.days) {
        const node = day.nodes.find((n) => n.id === quizId)
        if (node) return node.title
      }
    }
    return quizId
  }

  const formatDueDate = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diffMs = date.getTime() - now.getTime()
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

    if (diffDays < 0) {
      return `${Math.abs(diffDays)} day${Math.abs(diffDays) !== 1 ? 's' : ''} overdue`
    } else if (diffDays === 0) {
      return 'Due today'
    } else if (diffDays === 1) {
      return 'Due tomorrow'
    } else {
      return `Due in ${diffDays} days`
    }
  }

  const getDueUrgency = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diffMs = date.getTime() - now.getTime()
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

    if (diffDays < 0) {
      return 'text-red-600 bg-red-50'
    } else if (diffDays === 0) {
      return 'text-orange-600 bg-orange-50'
    } else {
      return 'text-blue-600 bg-blue-50'
    }
  }

  const handleStartReview = (review: ReviewItem) => {
    // Navigate to quiz page with review mode
    navigate(`/quiz/${review.quiz_id}?mode=review`)
  }

  if (loading && dueReviews.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Loading reviews...</p>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-3xl mx-auto">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Brain className="w-7 h-7 text-purple-500" />
          Review Queue
        </h1>
        <p className="text-gray-500 mt-1">
          Spaced repetition helps you remember what you've learned
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="bg-white rounded-xl p-4 shadow-sm border">
          <div className="flex items-center gap-2 text-gray-500 mb-1">
            <RotateCcw className="w-4 h-4" />
            <span className="text-sm">Due Now</span>
          </div>
          <p className="text-2xl font-bold text-purple-600">{dueCount}</p>
        </div>
        <div className="bg-white rounded-xl p-4 shadow-sm border">
          <div className="flex items-center gap-2 text-gray-500 mb-1">
            <Calendar className="w-4 h-4" />
            <span className="text-sm">Total Reviews</span>
          </div>
          <p className="text-2xl font-bold">{dueReviews.length}</p>
        </div>
        <div className="bg-white rounded-xl p-4 shadow-sm border">
          <div className="flex items-center gap-2 text-gray-500 mb-1">
            <AlertCircle className="w-4 h-4" />
            <span className="text-sm">Skills Decayed</span>
          </div>
          <p className="text-2xl font-bold text-orange-500">{decayedSkills}</p>
        </div>
      </div>

      {/* Mastery Decay Notice */}
      {decayedSkills > 0 && (
        <div className="mb-6 p-4 bg-orange-50 border border-orange-200 rounded-lg">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-orange-500 flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-semibold text-orange-800">Skills Need Attention</h3>
              <p className="text-sm text-orange-700 mt-1">
                {decayedSkills} skill{decayedSkills !== 1 ? 's have' : ' has'} decayed due to 
                inactivity. Complete reviews to restore your mastery!
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Due Reviews */}
      {dueReviews.length === 0 ? (
        <div className="text-center py-12">
          <CheckCircle2 className="w-16 h-16 text-green-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-700">All caught up!</h2>
          <p className="text-gray-500 mt-2">
            No reviews due right now. Keep learning to add more to your review queue.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          <h2 className="text-lg font-semibold mb-4">Reviews Due</h2>
          
          {dueReviews.map((review) => (
            <button
              key={review.quiz_id}
              onClick={() => handleStartReview(review)}
              className="w-full bg-white rounded-xl p-4 shadow-sm border hover:border-purple-300 
                       hover:shadow-md transition-all flex items-center justify-between text-left"
            >
              <div className="flex-1">
                <h3 className="font-semibold text-gray-800">
                  {getQuizTitle(review.quiz_id)}
                </h3>
                <div className="flex items-center gap-4 mt-2 text-sm">
                  <span className={`px-2 py-0.5 rounded-full ${getDueUrgency(review.due_date)}`}>
                    {formatDueDate(review.due_date)}
                  </span>
                  <span className="text-gray-400 flex items-center gap-1">
                    <Clock className="w-3.5 h-3.5" />
                    {review.repetitions} review{review.repetitions !== 1 ? 's' : ''}
                  </span>
                  <span className="text-gray-400">
                    Interval: {review.interval_days} day{review.interval_days !== 1 ? 's' : ''}
                  </span>
                </div>
              </div>
              <ChevronRight className="w-5 h-5 text-gray-400" />
            </button>
          ))}
        </div>
      )}

      {/* Info Card */}
      <div className="mt-8 p-4 bg-purple-50 border border-purple-100 rounded-lg">
        <h3 className="font-semibold text-purple-800 mb-2">How Spaced Repetition Works</h3>
        <ul className="text-sm text-purple-700 space-y-1">
          <li>• Reviews are scheduled based on your performance</li>
          <li>• Better scores = longer intervals between reviews</li>
          <li>• Struggling? The interval shortens to help you practice more</li>
          <li>• Regular reviews prevent skill decay over time</li>
        </ul>
      </div>
    </div>
  )
}
