import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface UpdateInfo {
  version: string
  current_version: string
  body: string | null
}

interface UpdateState {
  updateAvailable: boolean
  updateInfo: UpdateInfo | null
  downloading: boolean
  progress: number
  error: string | null
  lastChecked: Date | null

  checkForUpdate: () => Promise<void>
  downloadUpdate: () => Promise<void>
  dismissUpdate: () => void
  clearError: () => void
}

export const useUpdateStore = create<UpdateState>((set, get) => ({
  updateAvailable: false,
  updateInfo: null,
  downloading: false,
  progress: 0,
  error: null,
  lastChecked: null,

  checkForUpdate: async () => {
    try {
      set({ error: null })
      const info = await invoke<UpdateInfo | null>('check_for_update')
      set({
        updateAvailable: info !== null,
        updateInfo: info,
        lastChecked: new Date(),
      })
    } catch (error) {
      set({ error: String(error) })
    }
  },

  downloadUpdate: async () => {
    const { updateAvailable } = get()
    if (!updateAvailable) return

    try {
      set({ downloading: true, progress: 0, error: null })
      await invoke('download_and_install_update')
      // If we get here, the app will restart, so this won't execute
      set({ downloading: false, progress: 100 })
    } catch (error) {
      set({ downloading: false, error: String(error) })
    }
  },

  dismissUpdate: () => {
    set({ updateAvailable: false, updateInfo: null })
  },

  clearError: () => {
    set({ error: null })
  },
}))
