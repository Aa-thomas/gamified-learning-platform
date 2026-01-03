import { useEffect } from 'react'
import { BookOpen, Code, FileText, Lock, Check, Play } from 'lucide-react'
import { useContentStore } from '@/stores/contentStore'
import { useProgressStore } from '@/stores/progressStore'

export function SkillTree() {
  const { tree, fetchContentTree } = useContentStore()
  const { progress, fetchAllProgress, startNode } = useProgressStore()

  useEffect(() => {
    fetchContentTree()
    fetchAllProgress()
  }, [fetchContentTree, fetchAllProgress])

  if (!tree) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Loading skill tree...</p>
      </div>
    )
  }

  const getNodeIcon = (type: string) => {
    switch (type) {
      case 'lecture':
        return <BookOpen size={16} />
      case 'quiz':
        return <FileText size={16} />
      case 'challenge':
        return <Code size={16} />
      default:
        return <BookOpen size={16} />
    }
  }

  const getNodeStatus = (nodeId: string) => {
    const p = progress[nodeId]
    if (!p) return 'available'
    return p.status
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 border-green-500 text-green-700'
      case 'in_progress':
        return 'bg-yellow-100 border-yellow-500 text-yellow-700'
      case 'locked':
        return 'bg-gray-100 border-gray-300 text-gray-400'
      default:
        return 'bg-white border-gray-300 text-gray-700 hover:border-primary'
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <Check size={14} />
      case 'in_progress':
        return <Play size={14} />
      case 'locked':
        return <Lock size={14} />
      default:
        return null
    }
  }

  const handleNodeClick = async (nodeId: string) => {
    const status = getNodeStatus(nodeId)
    if (status === 'available') {
      await startNode(nodeId)
    }
    // In a full implementation, this would navigate to the node content
  }

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Skill Tree</h1>

      <div className="space-y-8">
        {tree.weeks.map((week) => (
          <div key={week.id} className="bg-white rounded-lg shadow p-6">
            <h2 className="text-lg font-semibold mb-2">{week.title}</h2>
            <p className="text-sm text-gray-500 mb-4">{week.description}</p>

            <div className="space-y-4">
              {week.days.map((day) => (
                <div key={day.id} className="border-l-2 border-gray-200 pl-4">
                  <h3 className="text-sm font-medium text-gray-700 mb-2">
                    {day.title}
                  </h3>
                  <p className="text-xs text-gray-500 mb-3">{day.description}</p>

                  <div className="flex flex-wrap gap-2">
                    {day.nodes.map((node) => {
                      const status = getNodeStatus(node.id)
                      return (
                        <button
                          key={node.id}
                          onClick={() => handleNodeClick(node.id)}
                          disabled={status === 'locked'}
                          className={`
                            flex items-center gap-2 px-3 py-2 rounded-lg border-2 
                            text-sm font-medium transition-all
                            ${getStatusColor(status)}
                            ${status !== 'locked' ? 'cursor-pointer' : 'cursor-not-allowed'}
                          `}
                        >
                          {getNodeIcon(node.node_type)}
                          <span>{node.title}</span>
                          {getStatusIcon(status)}
                          <span className="text-xs opacity-60">
                            +{node.xp_reward} XP
                          </span>
                        </button>
                      )
                    })}
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>

      {tree.weeks.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <p>No content available yet.</p>
        </div>
      )}
    </div>
  )
}
