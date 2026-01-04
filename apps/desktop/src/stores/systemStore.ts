import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface SystemStatus {
  docker_installed: boolean
  docker_running: boolean
  api_key_set: boolean
  database_ok: boolean
}

export interface DockerStatus {
  installed: boolean
  running: boolean
  version: string | null
}

interface SystemState {
  status: SystemStatus | null
  dockerStatus: DockerStatus | null
  loading: boolean
  error: string | null

  checkSystemStatus: () => Promise<void>
  checkDockerStatus: () => Promise<DockerStatus>
  saveApiKey: (apiKey: string) => Promise<void>
  getApiKeyStatus: () => Promise<boolean>
  exportUserData: (path: string) => Promise<void>
  importUserData: (path: string) => Promise<void>
  resetAllProgress: () => Promise<void>
  isFirstLaunch: () => Promise<boolean>
  completeOnboarding: () => Promise<void>
  isOnboardingComplete: () => Promise<boolean>
}

export const useSystemStore = create<SystemState>((set) => ({
  status: null,
  dockerStatus: null,
  loading: false,
  error: null,

  checkSystemStatus: async () => {
    set({ loading: true, error: null })
    try {
      const status = await invoke<SystemStatus>('check_system_status')
      set({ status, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  checkDockerStatus: async () => {
    try {
      const dockerStatus = await invoke<DockerStatus>('check_docker_status')
      set({ dockerStatus })
      return dockerStatus
    } catch (error) {
      const fallback: DockerStatus = { installed: false, running: false, version: null }
      set({ dockerStatus: fallback })
      return fallback
    }
  },

  saveApiKey: async (apiKey: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('save_api_key', { apiKey })
      // Update status
      const status = await invoke<SystemStatus>('check_system_status')
      set({ status, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  getApiKeyStatus: async () => {
    try {
      return await invoke<boolean>('get_api_key_status')
    } catch {
      return false
    }
  },

  exportUserData: async (path: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('export_user_data', { path })
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  importUserData: async (path: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('import_user_data', { path })
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  resetAllProgress: async () => {
    set({ loading: true, error: null })
    try {
      await invoke('reset_all_progress')
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  isFirstLaunch: async () => {
    try {
      return await invoke<boolean>('is_first_launch')
    } catch {
      return true
    }
  },

  completeOnboarding: async () => {
    try {
      await invoke('complete_onboarding')
    } catch (error) {
      console.error('Failed to complete onboarding:', error)
    }
  },

  isOnboardingComplete: async () => {
    try {
      return await invoke<boolean>('is_onboarding_complete')
    } catch {
      return false
    }
  },
}))
