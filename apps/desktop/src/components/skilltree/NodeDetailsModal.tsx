import { useNavigate } from 'react-router-dom'
import { 
  X, 
  Clock, 
  Star, 
  Zap, 
  BookOpen, 
  Code, 
  FileText, 
  Lock,
  Play,
  Check,
  ChevronRight
} from 'lucide-react'
import { ContentNode } from '@/stores/contentStore'

interface NodeDetailsModalProps {
  node: ContentNode
  status: 'locked' | 'available' | 'in_progress' | 'completed'
  onClose: () => void
  onStart?: () => Promise<void>
}

export function NodeDetailsModal({ node, status, onClose, onStart }: NodeDetailsModalProps) {
  const navigate = useNavigate()

  const getNodeIcon = () => {
    switch (node.node_type) {
      case 'lecture':
        return <BookOpen className="w-6 h-6" />
      case 'quiz':
        return <FileText className="w-6 h-6" />
      case 'challenge':
        return <Code className="w-6 h-6" />
      default:
        return <BookOpen className="w-6 h-6" />
    }
  }

  const getDifficultyStars = () => {
    const difficulty = node.difficulty || 'medium'
    const starCount = {
      'easy': 1,
      'medium': 2,
      'hard': 3,
      'very_hard': 4,
    }[difficulty] || 2

    return (
      <div className="flex items-center gap-0.5">
        {[...Array(4)].map((_, i) => (
          <Star
            key={i}
            className={`w-4 h-4 ${i < starCount ? 'text-yellow-400 fill-yellow-400' : 'text-gray-300'}`}
          />
        ))}
      </div>
    )
  }

  const getStatusColor = () => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 text-green-700 border-green-300'
      case 'in_progress':
        return 'bg-yellow-100 text-yellow-700 border-yellow-300'
      case 'locked':
        return 'bg-gray-100 text-gray-500 border-gray-300'
      default:
        return 'bg-blue-100 text-blue-700 border-blue-300'
    }
  }

  const getStatusLabel = () => {
    switch (status) {
      case 'completed':
        return 'Completed'
      case 'in_progress':
        return 'In Progress'
      case 'locked':
        return 'Locked'
      default:
        return 'Available'
    }
  }

  const handleStartOrContinue = async () => {
    if (status === 'available' && onStart) {
      await onStart()
    }
    
    // Navigate based on node type
    switch (node.node_type) {
      case 'lecture':
        navigate(`/lecture/${node.id}`)
        break
      case 'quiz':
        navigate(`/quiz/${node.id}`)
        break
      case 'challenge':
        navigate(`/challenge/${node.id}`)
        break
    }
    onClose()
  }

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div 
        className="bg-white rounded-2xl shadow-xl max-w-md w-full mx-4 overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="relative bg-gradient-to-br from-primary/10 to-primary/5 p-6 pb-4">
          <button
            onClick={onClose}
            className="absolute top-4 right-4 text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
          
          <div className="flex items-start gap-4">
            <div className="p-3 bg-white rounded-xl shadow-sm">
              {getNodeIcon()}
            </div>
            <div className="flex-1 min-w-0">
              <span className={`inline-block px-2 py-0.5 text-xs font-medium rounded-full border ${getStatusColor()} mb-2`}>
                {getStatusLabel()}
              </span>
              <h2 className="text-xl font-bold text-gray-800 leading-tight">
                {node.title}
              </h2>
              <p className="text-sm text-gray-500 capitalize mt-1">
                {node.node_type}
              </p>
            </div>
          </div>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-3 gap-4 p-4 border-b">
          <div className="text-center">
            <div className="flex items-center justify-center gap-1 text-purple-600 mb-1">
              <Zap className="w-4 h-4" />
              <span className="font-bold">{node.xp_reward}</span>
            </div>
            <p className="text-xs text-gray-500">XP Reward</p>
          </div>
          <div className="text-center">
            <div className="flex items-center justify-center gap-1 text-gray-600 mb-1">
              <Clock className="w-4 h-4" />
              <span className="font-bold">{node.estimated_minutes}</span>
            </div>
            <p className="text-xs text-gray-500">Minutes</p>
          </div>
          <div className="text-center">
            <div className="flex items-center justify-center mb-1">
              {getDifficultyStars()}
            </div>
            <p className="text-xs text-gray-500 capitalize">{node.difficulty || 'Medium'}</p>
          </div>
        </div>

        {/* Skills */}
        {node.skills && node.skills.length > 0 && (
          <div className="p-4 border-b">
            <h3 className="text-sm font-semibold text-gray-700 mb-2">Skills Trained</h3>
            <div className="flex flex-wrap gap-2">
              {node.skills.map((skill) => (
                <span
                  key={skill}
                  className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded-full"
                >
                  {skill}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Prerequisites */}
        {node.prerequisites && node.prerequisites.length > 0 && (
          <div className="p-4 border-b">
            <h3 className="text-sm font-semibold text-gray-700 mb-2">Prerequisites</h3>
            <div className="space-y-1">
              {node.prerequisites.map((prereq) => (
                <div
                  key={prereq}
                  className="flex items-center gap-2 text-sm text-gray-600"
                >
                  <ChevronRight className="w-4 h-4 text-gray-400" />
                  {prereq}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Action */}
        <div className="p-4">
          {status === 'locked' ? (
            <button
              disabled
              className="w-full py-3 bg-gray-100 text-gray-400 font-semibold rounded-xl 
                       flex items-center justify-center gap-2 cursor-not-allowed"
            >
              <Lock className="w-5 h-5" />
              Complete Prerequisites First
            </button>
          ) : status === 'completed' ? (
            <button
              onClick={handleStartOrContinue}
              className="w-full py-3 bg-green-50 text-green-600 font-semibold rounded-xl 
                       hover:bg-green-100 transition-colors flex items-center justify-center gap-2"
            >
              <Check className="w-5 h-5" />
              Review Again
            </button>
          ) : (
            <button
              onClick={handleStartOrContinue}
              className="w-full py-3 bg-primary text-white font-semibold rounded-xl 
                       hover:bg-primary/90 transition-colors flex items-center justify-center gap-2
                       shadow-lg hover:shadow-xl"
            >
              <Play className="w-5 h-5" />
              {status === 'in_progress' ? 'Continue' : 'Start'}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
