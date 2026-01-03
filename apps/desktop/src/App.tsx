import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Navigation } from './components/layout/Navigation'
import { StatusBar } from './components/layout/StatusBar'
import { Home } from './pages/Home'
import { Lecture } from './pages/Lecture'
import { Quiz } from './pages/Quiz'
import { SkillTree } from './pages/SkillTree'
import { Progress } from './pages/Progress'
import { Settings } from './pages/Settings'

export default function App() {
  return (
    <BrowserRouter>
      <div className="flex flex-col h-screen">
        <Navigation />

        <main className="flex-1 overflow-y-auto">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/lecture/:lectureId" element={<Lecture />} />
            <Route path="/quiz/:quizId" element={<Quiz />} />
            <Route path="/skill-tree" element={<SkillTree />} />
            <Route path="/progress" element={<Progress />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </main>

        <StatusBar />
      </div>
    </BrowserRouter>
  )
}
