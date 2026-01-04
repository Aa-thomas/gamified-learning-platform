import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface ReviewItem {
  quiz_id: string
  due_date: string
  ease_factor: number
  interval_days: number
  repetitions: number
  last_reviewed_at: string | null
}

interface ReviewState {
  dueReviews: ReviewItem[]
  dueCount: number
  loading: boolean
  error: string | null
  fetchDueReviews: () => Promise<void>
  fetchDueCount: () => Promise<void>
  submitReview: (quizId: string, scorePercentage: number) => Promise<ReviewItem>
  createReviewItem: (quizId: string) => Promise<ReviewItem>
  applyDecayOnStartup: () => Promise<number>
}

export const useReviewStore = create<ReviewState>((set, get) => ({
  dueReviews: [],
  dueCount: 0,
  loading: false,
  error: null,

  fetchDueReviews: async () => {
    set({ loading: true, error: null })
    try {
      const dueReviews = await invoke<ReviewItem[]>('get_due_reviews')
      set({ dueReviews, dueCount: dueReviews.length, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  fetchDueCount: async () => {
    try {
      const dueCount = await invoke<number>('get_due_review_count')
      set({ dueCount })
    } catch (error) {
      console.error('Failed to fetch due count:', error)
    }
  },

  submitReview: async (quizId: string, scorePercentage: number) => {
    set({ loading: true, error: null })
    try {
      const updated = await invoke<ReviewItem>('submit_review', {
        quizId,
        scorePercentage,
      })
      // Refresh due reviews
      await get().fetchDueReviews()
      return updated
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },

  createReviewItem: async (quizId: string) => {
    try {
      const item = await invoke<ReviewItem>('create_review_item', { quizId })
      return item
    } catch (error) {
      console.error('Failed to create review item:', error)
      throw error
    }
  },

  applyDecayOnStartup: async () => {
    try {
      const decayedCount = await invoke<number>('apply_mastery_decay_on_startup')
      return decayedCount
    } catch (error) {
      console.error('Failed to apply mastery decay:', error)
      return 0
    }
  },
}))
