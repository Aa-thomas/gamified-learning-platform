import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface User {
  id: number
  username: string
  xp: number
  level: number
  streak_days: number
  last_active_date: string
  created_at: string
}

interface UserState {
  user: User | null
  loading: boolean
  error: string | null
  fetchUser: () => Promise<void>
  createUser: (username: string) => Promise<void>
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

  createUser: async (username: string) => {
    set({ loading: true, error: null })
    try {
      const user = await invoke<User>('create_user', { username })
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
