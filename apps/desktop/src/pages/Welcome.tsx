import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useOnboardingStore, OnboardingStep } from '@/stores/onboardingStore'
import { useSystemStore } from '@/stores/systemStore'
import { useUserStore } from '@/stores/userStore'
import { Button } from '@/components/common/Button'
import {
  Rocket,
  CheckCircle,
  XCircle,
  ArrowRight,
  ArrowLeft,
  ExternalLink,
} from 'lucide-react'

export function Welcome() {
  const navigate = useNavigate()
  const {
    step,
    nextStep,
    previousStep,
    dockerReady,
    setDockerReady,
    setDockerSkipped,
    setApiKeySet,
    setApiKeySkipped,
    setProfileCreated,
  } = useOnboardingStore()
  const { checkDockerStatus, saveApiKey, completeOnboarding } = useSystemStore()
  const { createUser } = useUserStore()

  const [apiKey, setApiKey] = useState('')
  const [apiKeyLoading, setApiKeyLoading] = useState(false)
  const [apiKeyError, setApiKeyError] = useState<string | null>(null)
  const [dockerChecking, setDockerChecking] = useState(false)

  // Check Docker status on system-check step
  useEffect(() => {
    if (step === 'system-check') {
      checkDocker()
    }
  }, [step])

  const checkDocker = async () => {
    setDockerChecking(true)
    const status = await checkDockerStatus()
    setDockerReady(status.installed && status.running)
    setDockerChecking(false)
  }

  const handleApiKeySave = async () => {
    if (!apiKey.trim()) return
    setApiKeyLoading(true)
    setApiKeyError(null)
    try {
      await saveApiKey(apiKey.trim())
      setApiKeySet(true)
      nextStep()
    } catch (error) {
      setApiKeyError(String(error))
    } finally {
      setApiKeyLoading(false)
    }
  }

  const handleCreateProfile = async () => {
    try {
      await createUser('default')
      setProfileCreated(true)
      await completeOnboarding()
      nextStep()
    } catch (error) {
      console.error('Failed to create profile:', error)
    }
  }

  const handleFinish = () => {
    navigate('/')
  }

  const renderStep = () => {
    switch (step) {
      case 'welcome':
        return (
          <div className="text-center">
            <div className="w-20 h-20 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-6">
              <Rocket className="w-10 h-10 text-primary" />
            </div>
            <h1 className="text-3xl font-bold mb-4 dark:text-white">
              Welcome to RustCamp! 🦀
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mb-8 max-w-md mx-auto">
              A gamified learning platform for mastering Rust programming. Complete lessons,
              earn XP, unlock badges, and track your mastery.
            </p>
            <Button size="lg" onClick={nextStep}>
              Get Started <ArrowRight className="w-5 h-5 ml-2" />
            </Button>
          </div>
        )

      case 'system-check':
        return (
          <div>
            <h2 className="text-2xl font-bold mb-6 dark:text-white">System Check</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              Let's make sure your system is ready for the full experience.
            </p>

            <div className="space-y-4 mb-8">
              <div className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                {dockerChecking ? (
                  <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin" />
                ) : dockerReady ? (
                  <CheckCircle className="w-6 h-6 text-green-500" />
                ) : (
                  <XCircle className="w-6 h-6 text-yellow-500" />
                )}
                <div className="flex-1">
                  <p className="font-medium dark:text-white">Docker</p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {dockerChecking
                      ? 'Checking...'
                      : dockerReady
                      ? 'Docker is running'
                      : 'Docker is not running (code challenges will be disabled)'}
                  </p>
                </div>
                {!dockerReady && !dockerChecking && (
                  <Button variant="outline" size="sm" onClick={checkDocker}>
                    Retry
                  </Button>
                )}
              </div>
            </div>

            <div className="flex gap-3">
              <Button variant="outline" onClick={previousStep}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              <Button onClick={nextStep}>
                Continue <ArrowRight className="w-4 h-4 ml-2" />
              </Button>
            </div>
          </div>
        )

      case 'docker-setup':
        return (
          <div>
            <h2 className="text-2xl font-bold mb-6 dark:text-white">Docker Setup</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              Docker is required for running code challenges. Install Docker to get the full experience.
            </p>

            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 mb-6">
              <h3 className="font-medium mb-2 dark:text-white">Installation Instructions</h3>
              <div className="text-sm text-gray-600 dark:text-gray-400 space-y-2">
                <p><strong>Linux (Arch):</strong></p>
                <code className="block bg-gray-900 text-green-400 p-2 rounded text-xs">
                  sudo pacman -S docker && sudo systemctl start docker
                </code>
                <p className="mt-3"><strong>Linux (Ubuntu/Debian):</strong></p>
                <code className="block bg-gray-900 text-green-400 p-2 rounded text-xs">
                  sudo apt-get install docker.io && sudo systemctl start docker
                </code>
                <p className="mt-3"><strong>macOS/Windows:</strong></p>
                <a
                  href="https://docker.com/get-started"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline inline-flex items-center gap-1"
                >
                  Download Docker Desktop <ExternalLink className="w-3 h-3" />
                </a>
              </div>
            </div>

            <div className="flex gap-3">
              <Button variant="outline" onClick={previousStep}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              <Button variant="outline" onClick={checkDocker} loading={dockerChecking}>
                Check Again
              </Button>
              <Button
                variant="ghost"
                onClick={() => {
                  setDockerSkipped(true)
                  nextStep()
                }}
              >
                Skip for now
              </Button>
            </div>
          </div>
        )

      case 'api-key-setup':
        return (
          <div>
            <h2 className="text-2xl font-bold mb-6 dark:text-white">OpenAI API Key</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              An OpenAI API key is needed for AI-powered artifact grading. This is optional but
              recommended for the full experience.
            </p>

            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                API Key
              </label>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-..."
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary"
              />
              {apiKeyError && (
                <p className="text-sm text-red-500 mt-1">{apiKeyError}</p>
              )}
              <p className="text-xs text-gray-500 mt-2">
                Get your API key from{' '}
                <a
                  href="https://platform.openai.com/api-keys"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  platform.openai.com
                </a>
              </p>
            </div>

            <div className="flex gap-3">
              <Button variant="outline" onClick={previousStep}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              <Button
                onClick={handleApiKeySave}
                loading={apiKeyLoading}
                disabled={!apiKey.trim()}
              >
                Save & Continue
              </Button>
              <Button
                variant="ghost"
                onClick={() => {
                  setApiKeySkipped(true)
                  nextStep()
                }}
              >
                Skip for now
              </Button>
            </div>
          </div>
        )

      case 'profile':
        return (
          <div className="text-center">
            <h2 className="text-2xl font-bold mb-6 dark:text-white">Create Your Profile</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-8">
              You're all set! Let's create your learner profile and start your Rust journey.
            </p>
            <Button size="lg" onClick={handleCreateProfile}>
              Create Profile & Start Learning
            </Button>
          </div>
        )

      case 'tutorial':
      case 'complete':
        return (
          <div className="text-center">
            <div className="w-20 h-20 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
              <CheckCircle className="w-10 h-10 text-green-500" />
            </div>
            <h2 className="text-2xl font-bold mb-4 dark:text-white">You're All Set!</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-8">
              Your profile is ready. Start exploring the skill tree and begin your learning journey!
            </p>
            <Button size="lg" onClick={handleFinish}>
              Go to Dashboard <ArrowRight className="w-5 h-5 ml-2" />
            </Button>
          </div>
        )

      default:
        return null
    }
  }

  // Progress indicator
  const steps: OnboardingStep[] = ['welcome', 'system-check', 'api-key-setup', 'profile', 'complete']
  const currentIndex = steps.indexOf(step)

  return (
    <div className="min-h-screen flex flex-col items-center justify-center p-6 bg-gray-50 dark:bg-gray-900">
      {/* Progress dots */}
      <div className="flex gap-2 mb-8">
        {steps.map((s, i) => (
          <div
            key={s}
            className={`w-2 h-2 rounded-full transition-colors ${
              i <= currentIndex
                ? 'bg-primary'
                : 'bg-gray-300 dark:bg-gray-700'
            }`}
          />
        ))}
      </div>

      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 max-w-lg w-full">
        {renderStep()}
      </div>
    </div>
  )
}
