import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface BadgeDefinition {
  id: string
  name: string
  description: string
  icon: string
  threshold: number
  category: string
}

export interface BadgeWithProgress {
  definition: BadgeDefinition
  progress: number
  current_value: number
  is_earned: boolean
  earned_at: string | null
}

interface BadgeState {
  badges: BadgeWithProgress[]
  newlyUnlocked: BadgeDefinition[]
  loading: boolean
  error: string | null
  fetchAllBadges: () => Promise<void>
  checkAndUnlockBadges: () => Promise<BadgeDefinition[]>
  clearNewlyUnlocked: () => void
}

export const useBadgeStore = create<BadgeState>((set, get) => ({
  badges: [],
  newlyUnlocked: [],
  loading: false,
  error: null,

  fetchAllBadges: async () => {
    set({ loading: true, error: null })
    try {
      const badges = await invoke<BadgeWithProgress[]>('get_all_badges')
      set({ badges, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  checkAndUnlockBadges: async () => {
    try {
      const unlocked = await invoke<BadgeDefinition[]>('check_and_unlock_badges')
      if (unlocked.length > 0) {
        set({ newlyUnlocked: unlocked })
        // Refresh badge list
        await get().fetchAllBadges()
      }
      return unlocked
    } catch (error) {
      console.error('Failed to check badge unlocks:', error)
      return []
    }
  },

  clearNewlyUnlocked: () => {
    set({ newlyUnlocked: [] })
  },
}))
