import { useEffect, useState } from 'react'
import { BookOpen, Code, FileText, Lock, Check, Play, Map, Zap } from 'lucide-react'
import { useContentStore, ContentNode } from '@/stores/contentStore'
import { useProgressStore } from '@/stores/progressStore'
import { NodeDetailsModal } from '@/components/skilltree/NodeDetailsModal'

export function SkillTree() {
  const { tree, fetchContentTree } = useContentStore()
  const { progress, fetchAllProgress, startNode } = useProgressStore()
  const [selectedNode, setSelectedNode] = useState<ContentNode | null>(null)

  useEffect(() => {
    fetchContentTree()
    fetchAllProgress()
  }, [fetchContentTree, fetchAllProgress])

  if (!tree) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin w-8 h-8 border-4 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
          <p className="text-gray-500">Loading skill tree...</p>
        </div>
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

  const getNodeStatus = (nodeId: string): 'locked' | 'available' | 'in_progress' | 'completed' => {
    const p = progress[nodeId]
    if (!p) return 'available'
    return p.status
  }

  const getStatusStyles = (status: string) => {
    switch (status) {
      case 'completed':
        return {
          container: 'bg-green-50 border-green-400 text-green-700 hover:bg-green-100 hover:border-green-500',
          icon: 'text-green-500',
          xp: 'text-green-500',
        }
      case 'in_progress':
        return {
          container: 'bg-yellow-50 border-yellow-400 text-yellow-700 hover:bg-yellow-100 hover:border-yellow-500 animate-pulse-subtle',
          icon: 'text-yellow-500',
          xp: 'text-yellow-600',
        }
      case 'locked':
        return {
          container: 'bg-gray-100 border-gray-200 text-gray-400 cursor-not-allowed opacity-60',
          icon: 'text-gray-400',
          xp: 'text-gray-400',
        }
      default:
        return {
          container: 'bg-white border-gray-200 text-gray-700 hover:border-primary hover:shadow-md',
          icon: 'text-gray-600',
          xp: 'text-purple-500',
        }
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <Check size={14} className="text-green-500" />
      case 'in_progress':
        return <Play size={14} className="text-yellow-500" />
      case 'locked':
        return <Lock size={14} className="text-gray-400" />
      default:
        return null
    }
  }

  // Calculate progress stats
  const totalNodes = tree.weeks.reduce((sum, week) => 
    sum + week.days.reduce((daySum, day) => daySum + day.nodes.length, 0), 0)
  const completedNodes = Object.values(progress).filter(p => p.status === 'completed').length
  const progressPercentage = totalNodes > 0 ? Math.round((completedNodes / totalNodes) * 100) : 0

  return (
    <div className="p-6 max-w-5xl mx-auto">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Map className="w-7 h-7 text-primary" />
            Skill Tree
          </h1>
          <div className="flex items-center gap-4 text-sm">
            <span className="text-gray-500">
              {completedNodes} / {totalNodes} completed
            </span>
            <span className="font-semibold text-primary">{progressPercentage}%</span>
          </div>
        </div>
        
        {/* Overall progress bar */}
        <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
          <div 
            className="h-full bg-gradient-to-r from-primary to-purple-500 rounded-full transition-all duration-500"
            style={{ width: `${progressPercentage}%` }}
          />
        </div>
      </div>

      {/* Legend */}
      <div className="flex items-center gap-6 mb-6 text-sm">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-green-400"></div>
          <span className="text-gray-600">Completed</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-yellow-400"></div>
          <span className="text-gray-600">In Progress</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-blue-400"></div>
          <span className="text-gray-600">Available</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-gray-300"></div>
          <span className="text-gray-600">Locked</span>
        </div>
      </div>

      {/* Tree */}
      <div className="space-y-8">
        {tree.weeks.map((week, weekIndex) => (
          <div key={week.id} className="relative">
            {/* Week connector line */}
            {weekIndex < tree.weeks.length - 1 && (
              <div className="absolute left-6 top-full h-8 w-0.5 bg-gray-200 z-0"></div>
            )}
            
            <div className="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
              {/* Week header */}
              <div className="bg-gradient-to-r from-gray-50 to-white px-6 py-4 border-b">
                <h2 className="text-lg font-semibold text-gray-800">{week.title}</h2>
                <p className="text-sm text-gray-500 mt-1">{week.description}</p>
              </div>

              <div className="p-6 space-y-6">
                {week.days.map((day, dayIndex) => (
                  <div key={day.id} className="relative">
                    {/* Day connector */}
                    {dayIndex < week.days.length - 1 && (
                      <div className="absolute left-3 top-8 bottom-0 w-0.5 bg-gray-100"></div>
                    )}
                    
                    <div className="flex items-start gap-4">
                      {/* Day indicator */}
                      <div className="w-6 h-6 rounded-full bg-primary/10 border-2 border-primary/30 flex-shrink-0 mt-1"></div>
                      
                      <div className="flex-1 min-w-0">
                        <h3 className="text-sm font-semibold text-gray-700 mb-1">
                          {day.title}
                        </h3>
                        <p className="text-xs text-gray-500 mb-3">{day.description}</p>

                        <div className="flex flex-wrap gap-2">
                          {day.nodes.map((node) => {
                            const status = getNodeStatus(node.id)
                            const styles = getStatusStyles(status)
                            return (
                              <button
                                key={node.id}
                                onClick={() => status !== 'locked' && setSelectedNode(node)}
                                disabled={status === 'locked'}
                                className={`
                                  group flex items-center gap-2 px-3 py-2 rounded-lg border-2 
                                  text-sm font-medium transition-all duration-200
                                  ${styles.container}
                                `}
                              >
                                <span className={styles.icon}>
                                  {getNodeIcon(node.node_type)}
                                </span>
                                <span className="max-w-[150px] truncate">{node.title}</span>
                                {getStatusIcon(status)}
                                <span className={`flex items-center gap-0.5 text-xs ${styles.xp}`}>
                                  <Zap size={12} />
                                  {node.xp_reward}
                                </span>
                              </button>
                            )
                          })}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>

      {tree.weeks.length === 0 && (
        <div className="text-center py-12 bg-gray-50 rounded-xl border-2 border-dashed border-gray-200">
          <Map className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">No content available yet.</p>
          <p className="text-sm text-gray-400 mt-1">Check back soon for new learning content!</p>
        </div>
      )}

      {/* Node Details Modal */}
      {selectedNode && (
        <NodeDetailsModal
          node={selectedNode}
          status={getNodeStatus(selectedNode.id)}
          onClose={() => setSelectedNode(null)}
          onStart={() => startNode(selectedNode.id)}
        />
      )}

      {/* CSS for subtle pulse animation */}
      <style>{`
        .animate-pulse-subtle {
          animation: pulse-subtle 2s ease-in-out infinite;
        }
        
        @keyframes pulse-subtle {
          0%, 100% {
            box-shadow: 0 0 0 0 rgba(251, 191, 36, 0.4);
          }
          50% {
            box-shadow: 0 0 0 4px rgba(251, 191, 36, 0);
          }
        }
      `}</style>
    </div>
  )
}
