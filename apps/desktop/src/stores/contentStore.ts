import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface ContentNode {
  id: string
  title: string
  node_type: string
  xp_reward: number
  estimated_minutes: number
  content_path: string
  quiz_path?: string
  challenge_path?: string
  prerequisites: string[]
  skills: string[]
  difficulty?: string
}

export interface ContentTree {
  weeks: Week[]
}

export interface Week {
  id: string
  title: string
  description: string
  days: Day[]
}

export interface Day {
  id: string
  title: string
  description: string
  nodes: ContentNode[]
}

interface ContentState {
  tree: ContentTree | null
  currentNode: ContentNode | null
  loading: boolean
  error: string | null
  fetchContentTree: () => Promise<void>
  selectNode: (nodeId: string) => Promise<void>
}

export const useContentStore = create<ContentState>((set) => ({
  tree: null,
  currentNode: null,
  loading: false,
  error: null,

  fetchContentTree: async () => {
    set({ loading: true, error: null })
    try {
      const tree = await invoke<ContentTree>('get_content_tree')
      set({ tree, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  selectNode: async (nodeId: string) => {
    set({ loading: true, error: null })
    try {
      const node = await invoke<ContentNode>('get_node_by_id', { nodeId })
      set({ currentNode: node, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
}))
