import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { BookOpen, Code, FileText, ChevronRight } from 'lucide-react'
import { useUserStore } from '@/stores/userStore'
import { useContentStore } from '@/stores/contentStore'
import { useProgressStore } from '@/stores/progressStore'

export function Home() {
  const { user, createUser } = useUserStore()
  const { tree, fetchContentTree } = useContentStore()
  const { progress, fetchAllProgress } = useProgressStore()
  const [newUsername, setNewUsername] = useState('')

  useEffect(() => {
    fetchContentTree()
    fetchAllProgress()
  }, [fetchContentTree, fetchAllProgress])

  // New user onboarding
  if (!user) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="bg-white rounded-lg shadow-lg p-8 max-w-md w-full">
          <h1 className="text-2xl font-bold text-center mb-6">
            Welcome to RustCamp! 🦀
          </h1>
          <p className="text-gray-600 text-center mb-6">
            A gamified journey to mastering Rust programming.
          </p>
          <form
            onSubmit={(e) => {
              e.preventDefault()
              if (newUsername.trim()) {
                createUser(newUsername.trim())
              }
            }}
          >
            <input
              type="text"
              placeholder="Enter your username"
              value={newUsername}
              onChange={(e) => setNewUsername(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg mb-4 focus:outline-none focus:ring-2 focus:ring-primary"
            />
            <button
              type="submit"
              disabled={!newUsername.trim()}
              className="w-full bg-primary text-white py-2 rounded-lg font-medium hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Start Learning
            </button>
          </form>
        </div>
      </div>
    )
  }

  // Get next available node
  const getNextNode = () => {
    if (!tree) return null
    for (const week of tree.weeks) {
      for (const day of week.days) {
        for (const node of day.nodes) {
          const nodeProgress = progress[node.id]
          if (!nodeProgress || nodeProgress.status !== 'completed') {
            return { node, week, day }
          }
        }
      }
    }
    return null
  }

  const next = getNextNode()
  const completedCount = Object.values(progress).filter(
    (p) => p.status === 'completed'
  ).length
  const totalNodes = tree?.weeks.reduce(
    (sum, w) => sum + w.days.reduce((s, d) => s + d.nodes.length, 0),
    0
  ) ?? 0

  const getNodeIcon = (type: string) => {
    switch (type) {
      case 'lecture':
        return <BookOpen size={20} />
      case 'quiz':
        return <FileText size={20} />
      case 'challenge':
        return <Code size={20} />
      default:
        return <BookOpen size={20} />
    }
  }

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold mb-2">
        Welcome back, {user.username}! 👋
      </h1>
      <p className="text-gray-600 mb-8">
        Continue your Rust journey. You've completed {completedCount} of{' '}
        {totalNodes} lessons.
      </p>

      {/* Progress Overview */}
      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold mb-4">Your Progress</h2>
        <div className="w-full bg-gray-200 rounded-full h-4 mb-2">
          <div
            className="bg-primary h-4 rounded-full transition-all"
            style={{
              width: `${totalNodes > 0 ? (completedCount / totalNodes) * 100 : 0}%`,
            }}
          />
        </div>
        <p className="text-sm text-gray-500">
          {Math.round((completedCount / totalNodes) * 100 || 0)}% complete
        </p>
      </div>

      {/* Continue Learning */}
      {next && (
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold mb-4">Continue Learning</h2>
          <Link
            to="/skill-tree"
            className="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
          >
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center text-primary">
                {getNodeIcon(next.node.node_type)}
              </div>
              <div>
                <p className="font-medium">{next.node.title}</p>
                <p className="text-sm text-gray-500">
                  {next.week.title} → {next.day.title}
                </p>
              </div>
            </div>
            <ChevronRight className="text-gray-400" />
          </Link>
        </div>
      )}

      {!next && totalNodes > 0 && (
        <div className="bg-green-50 rounded-lg p-6 text-center">
          <p className="text-green-700 font-medium">
            🎉 Congratulations! You've completed all available content!
          </p>
        </div>
      )}
    </div>
  )
}
