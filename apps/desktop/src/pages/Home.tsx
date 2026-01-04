import { useEffect } from 'react'
import { Link } from 'react-router-dom'
import { BookOpen, Code, FileText, ChevronRight } from 'lucide-react'
import { useUserStore } from '@/stores/userStore'
import { useContentStore } from '@/stores/contentStore'
import { useProgressStore } from '@/stores/progressStore'

export function Home() {
  const { user, createUser } = useUserStore()
  const { tree, fetchContentTree } = useContentStore()
  const { progress, fetchAllProgress } = useProgressStore()

  useEffect(() => {
    fetchContentTree()
    fetchAllProgress()
  }, [fetchContentTree, fetchAllProgress])

  // New user onboarding
  if (!user) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 max-w-md w-full">
          <h1 className="text-2xl font-bold text-center mb-6 dark:text-white">
            Welcome to RustCamp! 🦀
          </h1>
          <p className="text-gray-600 dark:text-gray-400 text-center mb-6">
            A gamified journey to mastering Rust programming.
          </p>
          <button
            onClick={() => createUser()}
            className="w-full bg-primary text-white py-2 rounded-lg font-medium hover:bg-primary/90"
          >
            Start Learning
          </button>
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
      <h1 className="text-3xl font-bold mb-2 dark:text-white">
        Welcome back! 👋
      </h1>
      <p className="text-gray-600 dark:text-gray-400 mb-8">
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

      {totalNodes === 0 && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2 text-yellow-800 dark:text-yellow-200">
            No Curriculum Loaded
          </h2>
          <p className="text-yellow-700 dark:text-yellow-300 mb-4">
            Load a curriculum to start learning. Go to the Curriculum Manager to import or select a curriculum.
          </p>
          <Link
            to="/curriculum"
            className="inline-flex items-center gap-2 bg-yellow-600 text-white px-4 py-2 rounded-lg hover:bg-yellow-700 transition-colors"
          >
            Open Curriculum Manager <ChevronRight size={16} />
          </Link>
        </div>
      )}
    </div>
  )
}
