import { create } from 'zustand'

export type OnboardingStep =
  | 'welcome'
  | 'system-check'
  | 'docker-setup'
  | 'api-key-setup'
  | 'profile'
  | 'tutorial'
  | 'complete'

interface OnboardingState {
  step: OnboardingStep
  dockerReady: boolean
  dockerSkipped: boolean
  apiKeySet: boolean
  apiKeySkipped: boolean
  profileCreated: boolean
  tutorialCompleted: boolean

  setStep: (step: OnboardingStep) => void
  nextStep: () => void
  previousStep: () => void
  setDockerReady: (ready: boolean) => void
  setDockerSkipped: (skipped: boolean) => void
  setApiKeySet: (set: boolean) => void
  setApiKeySkipped: (skipped: boolean) => void
  setProfileCreated: (created: boolean) => void
  setTutorialCompleted: (completed: boolean) => void
  reset: () => void
}

const STEP_ORDER: OnboardingStep[] = [
  'welcome',
  'system-check',
  'docker-setup',
  'api-key-setup',
  'profile',
  'tutorial',
  'complete',
]

export const useOnboardingStore = create<OnboardingState>((set, get) => ({
  step: 'welcome',
  dockerReady: false,
  dockerSkipped: false,
  apiKeySet: false,
  apiKeySkipped: false,
  profileCreated: false,
  tutorialCompleted: false,

  setStep: (step) => set({ step }),

  nextStep: () => {
    const { step, dockerReady, apiKeySet } = get()
    const currentIndex = STEP_ORDER.indexOf(step)

    if (currentIndex < STEP_ORDER.length - 1) {
      let nextIndex = currentIndex + 1
      let nextStep = STEP_ORDER[nextIndex]

      // Skip docker-setup if docker is ready
      if (nextStep === 'docker-setup' && dockerReady) {
        nextIndex++
        nextStep = STEP_ORDER[nextIndex]
      }

      // Skip api-key-setup if already set
      if (nextStep === 'api-key-setup' && apiKeySet) {
        nextIndex++
        nextStep = STEP_ORDER[nextIndex]
      }

      set({ step: nextStep })
    }
  },

  previousStep: () => {
    const { step, dockerReady, apiKeySet } = get()
    const currentIndex = STEP_ORDER.indexOf(step)

    if (currentIndex > 0) {
      let prevIndex = currentIndex - 1
      let prevStep = STEP_ORDER[prevIndex]

      // Skip api-key-setup if already set
      if (prevStep === 'api-key-setup' && apiKeySet) {
        prevIndex--
        prevStep = STEP_ORDER[prevIndex]
      }

      // Skip docker-setup if docker is ready
      if (prevStep === 'docker-setup' && dockerReady) {
        prevIndex--
        prevStep = STEP_ORDER[prevIndex]
      }

      set({ step: prevStep })
    }
  },

  setDockerReady: (ready) => set({ dockerReady: ready }),
  setDockerSkipped: (skipped) => set({ dockerSkipped: skipped }),
  setApiKeySet: (isSet) => set({ apiKeySet: isSet }),
  setApiKeySkipped: (skipped) => set({ apiKeySkipped: skipped }),
  setProfileCreated: (created) => set({ profileCreated: created }),
  setTutorialCompleted: (completed) => set({ tutorialCompleted: completed }),

  reset: () =>
    set({
      step: 'welcome',
      dockerReady: false,
      dockerSkipped: false,
      apiKeySet: false,
      apiKeySkipped: false,
      profileCreated: false,
      tutorialCompleted: false,
    }),
}))
