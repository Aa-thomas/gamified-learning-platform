import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface User {
  id: string
  total_xp: number
  current_level: number
  current_streak: number
  xp_for_next_level: number
  xp_progress_percentage: number
}

interface UserState {
  user: User | null
  loading: boolean
  error: string | null
  fetchUser: () => Promise<void>
  createUser: () => Promise<void>
  updateXp: (xpDelta: number) => Promise<void>
}

export const useUserStore = create<UserState>((set, get) => ({
  user: null,
  loading: false,
  error: null,

  fetchUser: async () => {
    set({ loading: true, error: null })
    try {
      const user = await invoke<User | null>('get_user_data')
      set({ user, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  createUser: async () => {
    set({ loading: true, error: null })
    try {
      const user = await invoke<User>('create_user')
      set({ user, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  updateXp: async (xpDelta: number) => {
    const { user } = get()
    if (!user) return

    set({ loading: true, error: null })
    try {
      const updatedUser = await invoke<User>('update_user_xp', { xpDelta })
      set({ user: updatedUser, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
}))
