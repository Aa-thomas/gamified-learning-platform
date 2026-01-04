import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface ContentStats {
  total_weeks: number
  total_days: number
  total_nodes: number
  lectures: number
  quizzes: number
  challenges: number
  checkpoints: number
  total_xp: number
  total_estimated_minutes: number
}

export interface CurriculumInfo {
  id: string
  name: string
  version: string
  description: string | null
  author: string | null
  imported_at: string
  is_active: boolean
  stats: ContentStats | null
}

export interface ValidationResponse {
  is_valid: boolean
  errors: string[]
  warnings: string[]
  name: string | null
  version: string | null
  description: string | null
  author: string | null
  stats: ContentStats | null
}

export interface ImportResponse {
  success: boolean
  curriculum_id: string | null
  error: string | null
}

interface CurriculumState {
  curricula: CurriculumInfo[]
  activeCurriculum: CurriculumInfo | null
  loading: boolean
  error: string | null

  // Actions
  fetchCurricula: () => Promise<void>
  fetchActiveCurriculum: () => Promise<void>
  validateCurriculum: (sourcePath: string) => Promise<ValidationResponse>
  importCurriculum: (sourcePath: string, setActive: boolean) => Promise<ImportResponse>
  switchCurriculum: (curriculumId: string) => Promise<void>
  deleteCurriculum: (curriculumId: string, deleteProgress: boolean) => Promise<void>
}

export const useCurriculumStore = create<CurriculumState>((set, get) => ({
  curricula: [],
  activeCurriculum: null,
  loading: false,
  error: null,

  fetchCurricula: async () => {
    set({ loading: true, error: null })
    try {
      const curricula = await invoke<CurriculumInfo[]>('list_curricula')
      set({ curricula, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  fetchActiveCurriculum: async () => {
    set({ loading: true, error: null })
    try {
      const activeCurriculum = await invoke<CurriculumInfo | null>('get_active_curriculum')
      set({ activeCurriculum, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  validateCurriculum: async (sourcePath: string) => {
    set({ loading: true, error: null })
    try {
      const result = await invoke<ValidationResponse>('validate_curriculum', { sourcePath })
      set({ loading: false })
      return result
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  importCurriculum: async (sourcePath: string, setActive: boolean) => {
    set({ loading: true, error: null })
    try {
      const result = await invoke<ImportResponse>('import_curriculum', {
        sourcePath,
        setActive,
      })
      
      if (result.success) {
        // Refresh the list
        await get().fetchCurricula()
        if (setActive) {
          await get().fetchActiveCurriculum()
        }
      }
      
      set({ loading: false })
      return result
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  switchCurriculum: async (curriculumId: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('switch_curriculum', { curriculumId })
      await get().fetchActiveCurriculum()
      await get().fetchCurricula()
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  deleteCurriculum: async (curriculumId: string, deleteProgress: boolean) => {
    set({ loading: true, error: null })
    try {
      await invoke('delete_curriculum', { curriculumId, deleteProgress })
      await get().fetchCurricula()
      
      // If we deleted the active curriculum, refresh that too
      const { activeCurriculum } = get()
      if (activeCurriculum?.id === curriculumId) {
        set({ activeCurriculum: null })
      }
      
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },
}))
