import { BrowserRouter, Routes, Route, useLocation } from 'react-router-dom'
import { useEffect } from 'react'
import { Navigation } from './components/layout/Navigation'
import { StatusBar } from './components/layout/StatusBar'
import { BadgeUnlockNotification } from './components/badges/BadgeUnlockNotification'
import { KeyboardShortcutsModal } from './components/common/KeyboardShortcutsModal'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { useThemeStore } from './stores/themeStore'
import { Home } from './pages/Home'
import { Lecture } from './pages/Lecture'
import { Quiz } from './pages/Quiz'
import { SkillTree } from './pages/SkillTree'
import { Progress } from './pages/Progress'
import { Settings } from './pages/Settings'
import { Badges } from './pages/Badges'
import { Review } from './pages/Review'
import { CurriculumManager } from './pages/CurriculumManager'
import { Welcome } from './pages/Welcome'

function AppContent() {
  const location = useLocation()
  const isWelcomePage = location.pathname === '/welcome'
  const { shortcuts, isHelpModalOpen, closeHelpModal } = useKeyboardShortcuts()
  const theme = useThemeStore((state) => state.theme)

  // Apply theme to document
  useEffect(() => {
    const root = document.documentElement
    root.classList.remove('light', 'dark')

    if (theme === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
      root.classList.add(systemTheme)
    } else {
      root.classList.add(theme)
    }
  }, [theme])

  return (
    <div className="flex flex-col h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
      {!isWelcomePage && <Navigation />}

      <main className={`flex-1 overflow-y-auto ${!isWelcomePage ? '' : ''}`}>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/welcome" element={<Welcome />} />
          <Route path="/lecture/:lectureId" element={<Lecture />} />
          <Route path="/quiz/:quizId" element={<Quiz />} />
          <Route path="/skill-tree" element={<SkillTree />} />
          <Route path="/progress" element={<Progress />} />
          <Route path="/badges" element={<Badges />} />
          <Route path="/review" element={<Review />} />
          <Route path="/curriculum" element={<CurriculumManager />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>

      {!isWelcomePage && <StatusBar />}
      {!isWelcomePage && <BadgeUnlockNotification />}

      <KeyboardShortcutsModal
        isOpen={isHelpModalOpen}
        onClose={closeHelpModal}
        shortcuts={shortcuts}
      />
    </div>
  )
}

export default function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  )
}
