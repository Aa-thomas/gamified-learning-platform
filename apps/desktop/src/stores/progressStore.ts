import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface NodeProgress {
  id: number
  user_id: number
  node_id: string
  status: 'locked' | 'available' | 'in_progress' | 'completed'
  started_at?: string
  completed_at?: string
  xp_earned: number
  attempts: number
}

interface ProgressState {
  progress: Record<string, NodeProgress>
  loading: boolean
  error: string | null
  fetchAllProgress: () => Promise<void>
  startNode: (nodeId: string) => Promise<void>
  markComplete: (nodeId: string, xpEarned: number) => Promise<void>
}

export const useProgressStore = create<ProgressState>((set, get) => ({
  progress: {},
  loading: false,
  error: null,

  fetchAllProgress: async () => {
    set({ loading: true, error: null })
    try {
      const progressList = await invoke<NodeProgress[]>('get_all_progress')
      const progress: Record<string, NodeProgress> = {}
      for (const p of progressList) {
        progress[p.node_id] = p
      }
      set({ progress, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  startNode: async (nodeId: string) => {
    set({ loading: true, error: null })
    try {
      const nodeProgress = await invoke<NodeProgress>('start_node', { nodeId })
      const { progress } = get()
      set({
        progress: { ...progress, [nodeId]: nodeProgress },
        loading: false,
      })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  markComplete: async (nodeId: string, xpEarned: number) => {
    set({ loading: true, error: null })
    try {
      const nodeProgress = await invoke<NodeProgress>('mark_node_complete', {
        nodeId,
        xpEarned,
      })
      const { progress } = get()
      set({
        progress: { ...progress, [nodeId]: nodeProgress },
        loading: false,
      })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
}))
